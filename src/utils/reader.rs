use std::fs::File;
use std::io::Read;
use std::path::Path;

/// Returns the initial codepoint accumulator for the first byte.
/// The first byte is special, only want bottom 5 bits for width 2, 4 bits
/// for width 3, and 3 bits for width 4.
#[inline]
const fn utf8_first_byte(byte: u8, width: u32) -> u32 {
    (byte & (0x7F >> width)) as u32
}

/// Mask of the value bits of a continuation byte.
const CONT_MASK: u8 = 0b0011_1111;

/// Returns the value of `ch` updated with continuation byte `byte`.
#[inline]
const fn utf8_acc_cont_byte(ch: u32, byte: u8) -> u32 {
    (ch << 6) | (byte & CONT_MASK) as u32
}

trait StreamReader {
    type Item;

    fn consume(&mut self) -> Self::Item;
    fn collect(&mut self) -> Vec<Self::Item>;
}

/// Represents position in file.
/// Contains a index as a one-indexed tuple of (row, column) and a raw byte offset.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FileLoc {
    pub index: (usize, usize),
    pub offset: usize,
}

impl FileLoc {
    pub fn new(index: (usize, usize), offset: usize) -> Self {
        FileLoc { index, offset }
    }
}

/// An individual Unicode codepoint.
/// Can potentially the take form of a right-padded u32 if codepoint is invalid.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Codepoint {
    Valid(u32),
    Invalid(u32),
}

/// Any type of newline.
/// - `Newline::LF` is a line feed, or a UNIX newline `\n`.
/// - `Newline::CRLF` is a carriage return line feed, or a Windows newline `\r\n`.
/// - `Newline::CR` is a carriage return, or an old MacOS newline `\r`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Newline {
    LF,
    CRLF,
    CR,
}

/// A singular character: either a Valid character, an Invalid unicode codepoint,
/// a Newline, or a EOF.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CharAtom {
    Valid(char),
    Invalid(u32),
    Newline(Newline),
    EOF,
}

/// Reader utilized for accessing a file a codepoint at a time.
/// Double-buffered and works with lookahead.
struct CodepointReader {
    f: File,
    buf1: [u8; 4096],
    buf2: [u8; 4096],
    index: usize,
    total_offset: usize,
    partial: Option<usize>,
    secondary: bool,
    last: bool,
}

impl CodepointReader {
    /// Initializes a `CodepointReader` to read from a file.
    ///
    /// # Arguments
    /// - `path`: The path to the file to be read from.
    ///
    /// # Returns
    /// Either a `CodepointReader` or a `std::io::Error`
    /// if opening file and initial reads fail.
    fn new(path: &Path) -> Result<Self, std::io::Error> {
        let mut reader = CodepointReader {
            f: File::open(path)?,
            buf1: [0; 4096],
            buf2: [0; 4096],
            index: 0,
            total_offset: 0,
            partial: None,
            secondary: false,
            last: false,
        };
        let sz = reader.f.read(&mut reader.buf1)?;
        if sz != 4096 {
            reader.last = true;
            reader.partial = Some(sz);
            return Ok(reader);
        }
        let sz2 = reader.f.read(&mut reader.buf2)?;
        if sz2 != 4096 {
            reader.partial = Some(sz2);
            return Ok(reader);
        }
        Ok(reader)
    }

    /// Gets the next byte from the file.
    ///
    /// # Returns
    /// `u8` if there is another byte, `None` if there isn't.
    fn get_byte(&mut self) -> Option<u8> {
        if self.index == 4096 {
            if let Some(_) = self.partial {
                self.last = true;
            } else {
                let sz = match self.secondary {
                    false => self.f.read(&mut self.buf1).unwrap(),
                    true => self.f.read(&mut self.buf2).unwrap(),
                };
                if sz != 4096 {
                    self.partial = Some(sz)
                }
            }
            self.secondary = !self.secondary;
            self.index = 0;
        }

        let out = if self.last && self.index >= self.partial.unwrap_or(4096) {
            None
        } else if self.secondary {
            Some(self.buf2[self.index])
        } else {
            Some(self.buf1[self.index])
        };
        self.index += 1;
        if let Some(_) = out {
            self.total_offset += 1;
        }
        out
    }

    // TODO expand capabilities to support rest of multi code point sequences
    fn lookahead(&self) -> Option<u8> {
        if self.index == 4096 {
            match self.secondary {
                true => Some(self.buf1[0]),
                false => Some(self.buf2[0]),
            }
        } else {
            if self.index > self.partial.unwrap_or(4096) && self.last {
                return None;
            }
            match self.secondary {
                false => Some(self.buf1[self.index]),
                true => Some(self.buf2[self.index]),
            }
        }
    }
}

impl StreamReader for CodepointReader {
    type Item = Option<Codepoint>;

    /// Parses the next unicode codepoint into a `Codepoint`.
    /// If a codepoint is invalid/incomplete, will set the remaining bytes to 0.
    fn consume(&mut self) -> Self::Item {
        let x = match self.get_byte() {
            None => return None,
            Some(c) => c,
        };
        if x < 128 {
            return Some(Codepoint::Valid(x as u32));
        }

        let init = utf8_first_byte(x, 2);
        let y = match self.get_byte() {
            None => return Some(Codepoint::Invalid((x as u32) << 24)),
            Some(c) => c,
        };
        let mut ch = utf8_acc_cont_byte(init, y);
        if x >= 0xE0 {
            let z = match self.get_byte() {
                None => return Some(Codepoint::Invalid(((x as u32) << 24) | ((y as u32) << 16))),
                Some(c) => c,
            };
            let y_z = utf8_acc_cont_byte((y & CONT_MASK) as u32, z);
            ch = init << 12 | y_z;

            if x >= 0xF0 {
                let w = match self.get_byte() {
                    None => {
                        return Some(Codepoint::Invalid(
                            ((x as u32) << 24) | ((y as u32) << 16) | ((z as u32) << 8),
                        ))
                    }
                    Some(c) => c,
                };
                ch = (init & 7) << 18 | utf8_acc_cont_byte(y_z, w);
            }
        }
        Some(Codepoint::Valid(ch))
    }

    fn collect(&mut self) -> Vec<Self::Item> {
        let mut v = Vec::new();
        loop {
            let atom = self.consume();
            v.push(atom);
            if atom == None {
                break;
            }
        }
        v
    }
}

/// Reader for accessing a file a `CharAtom` at a time.
///
/// # Examples
/// ```
/// let path = std::path::Path::new("./test.c");
/// let r = AtomReader::new(&path).unwrap();
/// for i in r {
///	println!("{:?}", i);
///	if i == CharAtom::EOF {
///		break;
///	}
/// }
/// ```
pub struct AtomReader {
    reader: CodepointReader,
    pos: FileLoc,
}

impl AtomReader {
    /// Initializes a `AtomReader` to read from a file.
    ///
    /// # Arguments
    /// - `path`: The path to the file to be read from.
    ///
    /// # Returns
    /// Either a `AtomReader` or a `std::io::Error`
    /// if opening file and initial reads fail.
    pub fn new(path: &Path) -> Result<Self, std::io::Error> {
        Ok(AtomReader {
            reader: CodepointReader::new(path)?,
            pos: FileLoc::new((1, 0), 0),
        })
    }
}

impl StreamReader for AtomReader {
    type Item = (CharAtom, FileLoc);

    /// Fetches a tuple of the next `CharAtom` from the file and its location,
    /// which is itself a tuple of (row, column).
    fn consume(&mut self) -> Self::Item {
        self.pos.offset = self.reader.total_offset;
        let raw_codepoint = self.reader.consume();
        self.pos.index.1 += 1;
        let ch = match raw_codepoint {
            None => return (CharAtom::EOF, self.pos),
            Some(codepoint) => match codepoint {
                Codepoint::Invalid(c) => return (CharAtom::Invalid(c), self.pos),
                Codepoint::Valid(c) => c,
            },
        };
        let atom = match ch {
            0x0a => CharAtom::Newline(Newline::LF),
            0x0d => {
                if self.reader.lookahead().unwrap_or(0x00) == 0x0a {
                    self.reader.get_byte();
                    CharAtom::Newline(Newline::CRLF)
                } else {
                    CharAtom::Newline(Newline::CR)
                }
            }
            _ => CharAtom::Valid(std::char::from_u32(ch).unwrap()),
        };
        let out = (atom, self.pos);
        match atom {
            CharAtom::Newline(_) => {
                self.pos.index = (self.pos.index.0 + 1, 0);
            }
            _ => {}
        };
        out
    }

    fn collect(&mut self) -> Vec<Self::Item> {
        let mut v = Vec::new();
        loop {
            let atom = self.consume();
            v.push(atom);
            if atom.0 == CharAtom::EOF {
                break;
            }
        }
        v
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    /// Utility for making a temporary file.
    fn temp_file(p: &'static str, contents: &[u8]) -> &'static Path {
        let path = Path::new(p);
        let mut f = File::create(&path).unwrap();
        f.write_all(contents).unwrap();
        path
    }

    #[test]
    fn single_letter() {
        let path = temp_file("./ncc-test-single-letter", b"a");
        let mut reader = AtomReader::new(&path).unwrap();
        let expected = vec![
            (CharAtom::Valid('a'), FileLoc::new((1, 1), 0)),
            (CharAtom::EOF, FileLoc::new((1, 2), 1)),
        ];
        assert_eq!(reader.collect(), expected);
    }

    #[test]
    fn multi_letter() {
        let path = temp_file("./ncc-test-multi-letter", b"ab");
        let mut reader = AtomReader::new(&path).unwrap();
        let expected = vec![
            (CharAtom::Valid('a'), FileLoc::new((1, 1), 0)),
            (CharAtom::Valid('b'), FileLoc::new((1, 2), 1)),
            (CharAtom::EOF, FileLoc::new((1, 3), 2)),
        ];
        assert_eq!(reader.collect(), expected);
    }

    #[test]
    fn multi_letter_and_newline() {
        let path = temp_file("./ncc-test-multi-letter-and-newline", b"abc\na");
        let mut reader = AtomReader::new(&path).unwrap();
        let expected = vec![
            (CharAtom::Valid('a'), FileLoc::new((1, 1), 0)),
            (CharAtom::Valid('b'), FileLoc::new((1, 2), 1)),
            (CharAtom::Valid('c'), FileLoc::new((1, 3), 2)),
            (CharAtom::Newline(Newline::LF), FileLoc::new((1, 4), 3)),
            (CharAtom::Valid('a'), FileLoc::new((2, 1), 4)),
            (CharAtom::EOF, FileLoc::new((2, 2), 5)),
        ];
        assert_eq!(reader.collect(), expected);
    }

    #[test]
    fn carriage_return() {
        let path = temp_file("./ncc-test-carriage_return", b"a\rc");
        let mut reader = AtomReader::new(&path).unwrap();
        let expected = vec![
            (CharAtom::Valid('a'), FileLoc::new((1, 1), 0)),
            (CharAtom::Newline(Newline::CR), FileLoc::new((1, 2), 1)),
            (CharAtom::Valid('c'), FileLoc::new((2, 1), 2)),
            (CharAtom::EOF, FileLoc::new((2, 2), 3)),
        ];
        assert_eq!(reader.collect(), expected);
    }

    #[test]
    fn windows_return() {
        let path = temp_file("./ncc-test-windows-return", b"a\r\nc");
        let mut reader = AtomReader::new(&path).unwrap();
        let expected = vec![
            (CharAtom::Valid('a'), FileLoc::new((1, 1), 0)),
            (CharAtom::Newline(Newline::CRLF), FileLoc::new((1, 2), 1)),
            (CharAtom::Valid('c'), FileLoc::new((2, 1), 3)),
            (CharAtom::EOF, FileLoc::new((2, 2), 4)),
        ];
        assert_eq!(reader.collect(), expected);
    }

    #[test]
    fn broken_codepoints() {
        let path = temp_file("./ncc-test-broken-codepoint", &[0xF0]);
        let mut reader = AtomReader::new(&path).unwrap();
        let expected = vec![
            (CharAtom::Invalid(0xF0_00_00_00), FileLoc::new((1, 1), 0)),
            (CharAtom::EOF, FileLoc::new((1, 2), 1)),
        ];
        assert_eq!(reader.collect(), expected);

        let path = temp_file("./ncc-test-broken-codepoint-2b", &[0xF0, 0xAA]);
        let mut reader = AtomReader::new(&path).unwrap();
        let expected = vec![
            (CharAtom::Invalid(0xF0_AA_00_00), FileLoc::new((1, 1), 0)),
            (CharAtom::EOF, FileLoc::new((1, 2), 2)),
        ];
        assert_eq!(reader.collect(), expected);

        let path = temp_file("./ncc-test-broken-codepoint-3b", &[0xF0, 0xAA, 0xAA]);
        let mut reader = AtomReader::new(&path).unwrap();
        let expected = vec![
            (CharAtom::Invalid(0xF0_AA_AA_00), FileLoc::new((1, 1), 0)),
            (CharAtom::EOF, FileLoc::new((1, 2), 3)),
        ];
        assert_eq!(reader.collect(), expected);
    }
}

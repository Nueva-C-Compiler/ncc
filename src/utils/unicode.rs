use std::fs::File;
use std::path::Path;
use std::io::Read;

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

#[derive(Clone, Copy, Debug, PartialEq)]
enum Codepoint {
    Valid(u32),
    Invalid(u32),
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Newline {
    LF,
    CRLF,
    CR,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum CharAtom {
    Valid(char),
    Invalid(u32),
    Newline(Newline),
    EOF,
}

struct CodepointReader {
    f: File,
    buf1: [u8; 4096],
    buf2: [u8; 4096],
    index: usize,
    partial: Option<usize>,
    secondary: bool,
    last: bool
}

impl CodepointReader {
    fn new(path: &Path) -> Self {
        let mut reader = CodepointReader {
            f: File::open(path).unwrap(),
            buf1: [0; 4096],
            buf2: [0; 4096],
            index: 0,
            partial: None,
            secondary: false,
            last: false,
        };
        let sz = reader.f.read(&mut reader.buf1).unwrap();
        if sz != 4096 {
            reader.last = true;
            reader.partial = Some(sz);
            return reader;
        }
        let sz2 = reader.f.read(&mut reader.buf2).unwrap();
        if sz2 != 4096 {
            reader.partial = Some(sz2);
            return reader;
        }
        reader
    }

    fn get_byte(&mut self) -> Option<u8> {
        if self.index == 4096 {
            if let Some(_) = self.partial {
                self.last = true;
            } else {
                let sz = match self.secondary {
                    false => self.f.read(&mut self.buf1).unwrap(),
                    true => self.f.read(&mut self.buf2).unwrap(),
                };
                if sz != 4096 { self.partial = Some(sz) }
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

impl Iterator for CodepointReader {
    type Item = Codepoint;

    fn next(&mut self) -> Option<Self::Item> {
        let x = match self.get_byte() {
            None => return None,
            Some(c) => c,
        };
        if x < 128 { return Some(Codepoint::Valid(x as u32)); }

        let init = utf8_first_byte(x, 2);
        let y = match self.get_byte() {
            None => return Some(Codepoint::Invalid((x as u32) << 24)),
            Some(c) => {
                println!("Valid second char! {}", c);
                c
            }
        };
        let mut ch = utf8_acc_cont_byte(init, y);
        if x >= 0xE0 {
            let z = match self.get_byte() {
                None => {
                    return Some(Codepoint::Invalid(
                        ((x as u32) << 24) | ((y as u32) << 16)
                    ))
                },
                Some(c) => c,
            };
            let y_z = utf8_acc_cont_byte((y & CONT_MASK) as u32, z);
            ch = init << 12 | y_z;

            if x >= 0xF0 {
                let w = match self.get_byte() {
                    None => {
                        return Some(Codepoint::Invalid(
                            ((x as u32) << 24) | ((y as u32) << 16) | ((z as u32) << 8)
                        ))
                    },
                    Some(c) => c,
                };
                ch = (init & 7) << 18 | utf8_acc_cont_byte(y_z, w);
            }
        }
        Some(Codepoint::Valid(ch))
    }
}

struct AtomReader {
    reader: CodepointReader,
}

impl AtomReader {
    fn new(path: &Path) -> Self {
        AtomReader {
            reader: CodepointReader::new(path)
        }
    }
}

impl Iterator for AtomReader {
    type Item = CharAtom;

    fn next(&mut self) -> Option<Self::Item> {
        let raw_codepoint = self.reader.next();
        let ch = match raw_codepoint {
            None => return Some(CharAtom::EOF),
            Some(codepoint) => match codepoint {
                Codepoint::Invalid(c) => return Some(CharAtom::Invalid(c)),
                Codepoint::Valid(c) => c,
            }
        };
        return Some(match ch {
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
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn temp_file(p: &'static str, contents: &[u8]) -> &'static Path {
        let path = Path::new(p);
        let mut f = File::create(&path).unwrap();         
        f.write_all(contents).unwrap();
        path
    }

    #[test]
    fn single_letter() {
        let path = temp_file("/tmp/ncc-single-letter", b"a");
        let reader = AtomReader::new(&path);
        let expected = vec![
            CharAtom::Valid('a' as char),
            CharAtom::EOF,
        ];
        assert_eq!(reader.take(expected.len()).collect::<Vec<CharAtom>>(), expected);
    }

    #[test]
    fn multi_letter() {
        let path = temp_file("/tmp/ncc-multi-letter", b"ab");
        let reader = AtomReader::new(&path);
        let expected = vec![
            CharAtom::Valid('a' as char),
            CharAtom::Valid('b' as char),
            CharAtom::EOF,
        ];
        assert_eq!(reader.take(expected.len()).collect::<Vec<CharAtom>>(), expected);
    }

    #[test]
    fn multi_letter_and_newline() {
        let path = temp_file("/tmp/ncc-multi-letter-and-newline", b"abc\na");
        let reader = AtomReader::new(&path);
        let expected = vec![
            CharAtom::Valid('a' as char),
            CharAtom::Valid('b' as char),
            CharAtom::Valid('c' as char),
            CharAtom::Newline(Newline::LF),
            CharAtom::Valid('a' as char),
            CharAtom::EOF,
        ];
        assert_eq!(reader.take(expected.len()).collect::<Vec<CharAtom>>(), expected);
    }

    #[test]
    fn carriage_return() {
        let path = temp_file("/tmp/ncc-carriage_return", b"a\rc");
        let reader = AtomReader::new(&path);
        let expected = vec![
            CharAtom::Valid('a' as char),
            CharAtom::Newline(Newline::CR),
            CharAtom::Valid('c' as char),
            CharAtom::EOF,
        ];
        assert_eq!(reader.take(expected.len()).collect::<Vec<CharAtom>>(), expected);
    }

    #[test]
    fn windows_return() {
        let path = temp_file("/tmp/ncc-windows-return", b"a\r\nc");
        let reader = AtomReader::new(&path);
        let expected = vec![
            CharAtom::Valid('a' as char),
            CharAtom::Newline(Newline::CRLF),
            CharAtom::Valid('c' as char),
            CharAtom::EOF,
        ];
        assert_eq!(reader.take(expected.len()).collect::<Vec<CharAtom>>(), expected);
    }

    #[test]
    fn broken_codepoints() {
        let path = temp_file("/tmp/ncc-broken-codepoint", &[0xF0]);
        let reader = AtomReader::new(&path);
        let expected = vec![
            CharAtom::Invalid(0xF0_00_00_00),
            CharAtom::EOF,
        ];
        assert_eq!(reader.take(expected.len()).collect::<Vec<CharAtom>>(), expected);

        let path = temp_file("/tmp/ncc-broken-codepoint-2b", &[0xF0, 0xAA]);
        let reader = AtomReader::new(&path);
        let expected = vec![
            CharAtom::Invalid(0xF0_AA_00_00),
            CharAtom::EOF,
        ];
        assert_eq!(reader.take(expected.len()).collect::<Vec<CharAtom>>(), expected);

        let path = temp_file("/tmp/ncc-broken-codepoint-3b", &[0xF0, 0xAA, 0xAA]);
        let reader = AtomReader::new(&path);
        let expected = vec![
            CharAtom::Invalid(0xF0_AA_AA_00),
            CharAtom::EOF,
        ];
        assert_eq!(reader.take(expected.len()).collect::<Vec<CharAtom>>(), expected);
    }
}

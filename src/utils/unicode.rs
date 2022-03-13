use std::fs::{File};
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

#[derive(Clone, Copy, Debug)]
enum Codepoint {
    Valid(u32),
    Invalid(u32),
}

#[derive(Clone, Copy, Debug)]
enum Newline {
    LF,
    CRLF,
    CR,
}

#[derive(Clone, Copy, Debug)]
enum CharAtom {
    Valid(char),
    Invalid(u32),
    Newline(Newline),
    EOF,
}

struct CodepointReader {
    f: File,
    buf: [u8; 4] // TODO: Dubious
}

impl Iterator for CodepointReader {
    type Item = Codepoint;

    // Basically lifted from std::core::str.
    fn next(&mut self) -> Option<Self::Item> {
        let sz = self.f.read(&mut self.buf[..1]).unwrap();
        if sz == 0 { return None; }
        else if self.buf[0] < 128 {
            return Some(Codepoint::Valid(self.buf[0] as u32));
        }

        let init = utf8_first_byte(self.buf[0], 2);
        let sz = self.f.read(&mut self.buf[1..2]).unwrap();
        if sz == 0 { return Some(Codepoint::Invalid((self.buf[0] as u32) << 24)); }
        let mut ch = utf8_acc_cont_byte(init, self.buf[1]);

        if self.buf[0] >= 0xE0 {
            let sz = self.f.read(&mut self.buf[2..3]).unwrap();
            if sz == 0 {
                return Some(Codepoint::Invalid(
                    ((self.buf[0] as u32) << 24) | ((self.buf[1] as u32) << 16)
                ))
            }
            let y_z = utf8_acc_cont_byte((self.buf[1] & CONT_MASK) as u32, self.buf[2]);
            ch = init << 12 | y_z;

            if self.buf[0] >= 0xF0 {
               let sz = self.f.read(&mut self.buf[3..]).unwrap();
                if sz == 0 {
                    return Some(Codepoint::Invalid(
                        ((self.buf[0] as u32) << 24) | ((self.buf[1] as u32) << 16)
                            | ((self.buf[2] as u32) << 8)
                    ))
                }
                ch = (init & 7) << 18 | utf8_acc_cont_byte(y_z, self.buf[3]);
            }
        }
        self.buf = [0; 4];
        Some(Codepoint::Valid(ch))
    }
}

struct AtomReader {
    reader: CodepointReader,
}

impl Iterator for AtomReader {
    type Item = CharAtom;

    fn next(&mut self) -> Option<Self::Item> {
        let raw_codepoint = self.reader.next();
        // TODO: Handle \r\n
        // TODO: Unicode validity checks
        match raw_codepoint.unwrap() {
            Codepoint::Valid(c) => {
                match c {
                    0x0a => Some(CharAtom::Newline(Newline::LF)),
                    0x0d => Some(CharAtom::Newline(Newline::CR)),
                    _ => None,
                }
            }
            _ => None,
        }
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn simple_newline() {

//     }
// }

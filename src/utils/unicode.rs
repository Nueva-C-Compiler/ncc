use std::fs::File;

#[derive(Copy, Debug)]
enum Newline {
    LF,
    CRLF,
    CR,
}

#[derive(Copy, Debug)]
enum CharAtom {
    Valid(char),
    Invalid(u16),
    Newline(Newline),
    EOF,
}

struct CodepointReader {
    f: File,
    buf: [u16; 1], // TODO: Dubious
}

impl Iterator for CodepointReader {
    type Item = u16;

    fn next(&mut self) -> Option<Self::Item> {
        self.f.read();
        buf[0]
    }
}

struct AtomReader {
    reader: CodepointReader,
}

impl Iterator for AtomReader {
    type Item = Atom;

    fn next(&mut self) -> Option<Self::Item> {
        let raw_codepoint = self.reader.next();
        // TODO: Handle \r\n
        // TODO: Unicode validity checks
        match raw_codepoint {
            0x0a => Newline(Newline::LF),
            0x0d => Newline(Newline::CR),
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

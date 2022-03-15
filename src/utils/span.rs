use crate::utils::reader::FileLoc;
use std::fs::File;
use positioned_io::ReadAt;

#[derive(Clone, Debug)]
pub struct Span {
    pub path: String,
    pub begin: FileLoc,
    pub end: FileLoc,
}

impl Span {
    pub fn new(path: String, begin: FileLoc, end: FileLoc) -> Self {
	Span {
	    path,
	    begin,
	    end
	}
    }       
}

impl std::fmt::Display for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
	let file = File::open(&self.path).unwrap();
	let mut v = vec![0; self.end.offset - self.begin.offset];
	file.read_at(self.begin.offset as u64, &mut v).unwrap();
	write!(f, "{}", format!("{}", String::from_utf8(v.clone()).unwrap()))
    }
}

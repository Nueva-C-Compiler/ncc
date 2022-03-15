pub mod utils;
use utils::span::Span;

use crate::utils::reader::FileLoc;

fn main() {
    let s = Span::new(
	"./test.c".to_string(),
	FileLoc::new((0,0), 4),
	FileLoc::new((0,0), 12),
    );
    println!("{}", s);
}

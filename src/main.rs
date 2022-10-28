mod tadaitem;
mod tadalist;
//use crate::tadaitem::*;
use crate::tadalist::*;

use std::fs::File;

fn main() {
	let testfile = File::open("example-todo.txt");
	let l = TadaList::new_from_file(testfile.unwrap());

	println!("{:?}", l);
}

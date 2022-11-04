mod item;
mod list;
//use crate::item::*;
use crate::list::*;

use std::fs::File;

fn main() {
	let testfile = File::open("example-todo.txt");
	let l = List::new_from_file(testfile.unwrap());

	println!("{:?}", l.items());
}

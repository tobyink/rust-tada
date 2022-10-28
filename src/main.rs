mod tadaitem;
mod tadalist;
//use crate::tadaitem::*;
use crate::tadalist::*;

use std::fs::File;

fn main() {
	/*
	let i = TadaItem::parse("x (A) 2000-01-01 Foo bar @baz");
	println!("{:?}", i);
	println!("{}", i);
	*/

	let foo = File::open("foo.txt");
	let l = TadaList::new_from_file(foo.unwrap());

	println!("{:?}", l);
}

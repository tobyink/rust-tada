mod item;
mod list;
use crate::item::*;
use crate::list::*;

use std::fs::File;
use std::io;

fn main() {
	let testfile = File::open("example-todo.txt");
	let l = List::new_from_file(testfile.unwrap());

	let mut out = io::stdout();
	let mut cfg = ItemFormatConfig::new();
	cfg.colour = true;

	for i in l.items() {
		i.write_to(&mut out, &cfg);
	}
}

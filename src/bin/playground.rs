use tada::*;
use std::fs::File;
use std::io;
use std::io::Write;

fn main() {
	let testfile = File::open("example-todo.txt");
	let l = List::new_from_file(testfile.unwrap());

	let split = Item::split_by_urgency(l.items());
	let mut out = io::stdout();
	let mut cfg = ItemFormatConfig::new();
	cfg.colour = true;

	for u in URGENCIES.iter() {
		if let Some(items) = split.get(u) {
			writeln!(&out, "{:?}:-", u).expect("panik");
			for i in Item::preferred_sort(items.to_vec()).iter() {
				i.write_to(&mut out, &cfg);
			}
			writeln!(&out).expect("panik");
		}
	}
}

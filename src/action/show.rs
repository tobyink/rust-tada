use crate::*;
use clap::{ArgMatches, Command};
use std::fs::File;
use std::io;
use std::io::Write;

pub fn get_action() -> Action {
	let name = String::from("show");
	let mut command = Command::new("show").about("Show the full todo list");

	command = Action::_add_file_options(command);

	Action { name, command }
}

pub fn execute(args: &ArgMatches) {
	let default = String::from("example-todo.txt");
	let filename = args
		.get_one::<String>("file")
		.unwrap_or(&default);

	let testfile = File::open(filename);
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

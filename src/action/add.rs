use crate::*;
use clap::{Arg, ArgMatches, Command};
use std::fs::OpenOptions;
use std::io;
use std::io::Write;

/// Options for the `add` subcommand.
pub fn get_action() -> Action {
	let name = String::from("add");
	let mut command = Command::new("add")
		.about("Add an item to the todo list")
		.after_help("After success, displays the added task.")
		.arg(Arg::new("task").help("Task text (may use todo.txt features)"));

	command = Action::_add_todotxt_file_options(command);
	command = command.arg(
		Arg::new("no-date")
			.num_args(0)
			.long("no-date")
			.aliases(["nodate"])
			.help("Don't automatically add a creation date to the task"),
	);
	command = Action::_add_output_options(command);

	Action { name, command }
}

/// Execute the `add` subcommand.
pub fn execute(args: &ArgMatches) {
	let input = args.get_one::<String>("task").unwrap();
	let mut item = Item::parse(input);

	let no_date = *args.get_one::<bool>("no-date").unwrap();
	if item.creation_date.is_none() && !no_date {
		item.creation_date = Some(chrono::Utc::now().date_naive());
	}

	let filename = Action::determine_filename(FileType::TodoTxt, args);
	let mut file = OpenOptions::new()
		.write(true)
		.append(true)
		.open(filename)
		.unwrap();
	if let Err(e) = writeln!(file, "{}", item) {
		eprintln!("Couldn't write to file: {}", e);
	} else {
		let cfg = Action::build_output_config(args);
		let mut out = io::stdout();
		item.write_to(&mut out, &cfg);
	}
}

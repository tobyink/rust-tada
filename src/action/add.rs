use crate::action::{Action, FileType};
use crate::item::Item;
use crate::list::{Line, List};
use clap::{Arg, ArgMatches, Command};
use std::io;

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
	if item.creation_date().is_none() && !no_date {
		item.set_creation_date(chrono::Utc::now().date_naive());
	}

	let line = Line::from_item(item);
	let filename = Action::determine_filename(FileType::TodoTxt, args);
	List::append_lines_to_url(filename, Vec::from([&line]));

	let cfg = Action::build_output_config(args);
	let mut out = io::stdout();
	line.item.unwrap().write_to(&mut out, &cfg);
}

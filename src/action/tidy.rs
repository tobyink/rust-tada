use crate::action::{Action, FileType};
use crate::list::{Line, List};
use crate::util::*;
use clap::{Arg, ArgMatches, Command};

/// Options for the `tidy` subcommand.
pub fn get_action() -> Action {
	let name = String::from("tidy");
	let mut command = Command::new("tidy").about("Remove blank lines and comments from a todo list")
		.after_help("This is the only command which will renumber tasks in your todo list.");

	command = Action::_add_todotxt_file_options(command);
	command = command
		.arg(
			Arg::new("sort")
				.num_args(1)
				.short('s')
				.long("sort")
				.value_name("BY")
				.help("sort by 'smart', 'urgency', 'importance', 'size', 'alpha', 'due', or 'orig' (default)"),
		);

	Action { name, command }
}

/// Execute the `tidy` subcommand.
pub fn execute(args: &ArgMatches) {
	let default_sort_by_type = String::from("orig");
	let sort_by_type = args
		.get_one::<String>("sort")
		.unwrap_or(&default_sort_by_type);

	let todo_filename = Action::determine_filename(FileType::TodoTxt, args);
	let list = List::from_url(todo_filename.clone())
		.expect("Could not read todo list");

	let mut new_list = List::new();
	for item in sort_items_by(sort_by_type.as_str(), list.items()) {
		new_list
			.lines
			.push(Line::from_item(item.clone()));
	}
	new_list.to_url(todo_filename);
}

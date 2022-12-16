//! Remove blank lines and comments from a todo list

use crate::action::*;
use clap::{ArgMatches, Command};

/// Options for the `tidy` subcommand.
pub fn get_action() -> Action {
	let name = String::from("tidy");
	let mut command = Command::new("tidy").about("Remove blank lines and comments from a todo list")
		.after_help("This is the only command which will renumber tasks in your todo list.");

	command = FileType::TodoTxt.add_args(command);
	command = SortOrder::add_args(command, default_sort_order());

	Action { name, command }
}

/// The default sort order for output.
pub fn default_sort_order() -> SortOrder {
	SortOrder::Original
}

/// Execute the `tidy` subcommand.
pub fn execute(args: &ArgMatches) {
	let todo_filename = FileType::TodoTxt.filename(args);
	let list = FileType::TodoTxt.load(args);
	let sort_order = SortOrder::from_argmatches(args, default_sort_order());

	list.but_tidy(&sort_order).to_url(todo_filename);
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_get_action() {
		assert_eq!(String::from("tidy"), get_action().name);
	}

	#[test]
	fn test_default_sort_order() {
		assert_eq!(SortOrder::Original, default_sort_order());
	}
}

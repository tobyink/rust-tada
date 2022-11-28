//! Show the smallest tasks

use crate::action::*;
use clap::{ArgMatches, Command};

/// Options for the `quick` subcommand.
pub fn get_action() -> Action {
	let name = String::from("quick");
	let mut command = Command::new("quick")
		.aliases(["q"])
		.about("Show the smallest tasks")
		.after_help(
			"Ignores tasks which are marked as already complete or \
			have a start date in the future.",
		);
	command = FileType::TodoTxt.add_args(command);
	command = Outputter::add_args(command);
	command = OutputCount::add_args(command);
	command = SortOrder::add_args(command, default_sort_order());
	Action { name, command }
}

/// The default sort order for output.
pub fn default_sort_order() -> SortOrder {
	SortOrder::TshirtSize
}

/// Execute the `important` subcommand.
#[cfg(not(tarpaulin_include))]
pub fn execute(args: &ArgMatches) {
	execute_simple_list_action(args, default_sort_order());
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_get_action() {
		assert_eq!(String::from("quick"), get_action().name);
	}

	#[test]
	fn test_default_sort_order() {
		assert_eq!(SortOrder::TshirtSize, default_sort_order());
	}
}

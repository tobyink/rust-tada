//! Show the most urgent tasks

use crate::action::*;
use clap::{ArgMatches, Command};

/// Options for the `urgent` subcommand.
pub fn get_action() -> Action {
	let name = String::from("urgent");
	let mut command = Command::new("urgent")
		.aliases(["u"])
		.about("Show the most urgent tasks")
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
	SortOrder::Urgency
}

/// Execute the `urgent` subcommand.
#[cfg(not(tarpaulin_include))]
pub fn execute(args: &ArgMatches) {
	execute_simple_list_action(args, default_sort_order());
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_get_action() {
		assert_eq!(String::from("urgent"), get_action().name);
	}

	#[test]
	fn test_default_sort_order() {
		assert_eq!(SortOrder::Urgency, default_sort_order());
	}
}

//! Prints the full path to your todo list

use crate::action::*;
use clap::{ArgMatches, Command};

/// Options for the `path` subcommand.
pub fn get_action() -> Action {
	let name = String::from("path");
	let mut command = Command::new("path")
		.about("Prints the full path to your todo list")
		.after_help(
			"This allows things like:\n\
			\n  /path/to/some/editor `tada path`",
		);
	command = FileType::TodoTxt.add_args(command);
	Action { name, command }
}

/// Execute the `path` subcommand.
#[cfg(not(tarpaulin_include))]
pub fn execute(args: &ArgMatches) {
	let f = FileType::TodoTxt.filename(args);
	println!("{}", f);
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_get_action() {
		assert_eq!(String::from("path"), get_action().name);
	}
}

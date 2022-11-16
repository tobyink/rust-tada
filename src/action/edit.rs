use crate::action::{Action, FileType};
use clap::{ArgMatches, Command};
use std::env;
use std::process::Command as SysCommand;

/// Options for the `edit` subcommand.
pub fn get_action() -> Action {
	let name = String::from("edit");
	let mut command = Command::new("edit")
		.about("Open your todo list in your editor")
		.after_help("Ensure the EDITOR environent variable is set.");
	command = Action::_add_todotxt_file_options(command);
	Action { name, command }
}

/// Execute the `edit` subcommand.
pub fn execute(args: &ArgMatches) {
	let filename = Action::determine_filename(FileType::TodoTxt, args);
	let editor =
		editor().unwrap_or_else(|_| panic!("Could not determine EDITOR"));

	let mut cmd = SysCommand::new(editor);
	let cmd = cmd.args([filename]);
	let mut child = cmd.spawn().unwrap();
	child.wait().unwrap();
}

pub fn editor() -> Result<String, env::VarError> {
	match env::var("EDITOR") {
		Ok(result) => return Ok(result),
		Err(env::VarError::NotPresent) => {}
		Err(error) => return Err(error),
	}
	Ok("vi".to_string())
}

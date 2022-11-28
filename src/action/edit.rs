//! Open your todo list in your editor

use crate::action::*;
use clap::{ArgMatches, Command};
use std::{env, io, process};

/// Options for the `edit` subcommand.
pub fn get_action() -> Action {
	let name = String::from("edit");
	let mut command = Command::new("edit")
		.about("Open your todo list in your editor")
		.after_help("Ensure the EDITOR environent variable is set.");
	command = FileType::TodoTxt.add_args(command);
	Action { name, command }
}

/// Execute the `edit` subcommand.
#[cfg(not(tarpaulin_include))]
pub fn execute(args: &ArgMatches) {
	let editor =
		editor().unwrap_or_else(|_| panic!("Could not determine EDITOR"));
	let filename = FileType::TodoTxt.filename(args);
	open_file_in_editor(editor, filename).unwrap();
}

/// Figure out the editor to use based on the environment.
pub fn editor() -> Result<String, env::VarError> {
	match env::var("EDITOR") {
		Ok(result) => return Ok(result),
		Err(env::VarError::NotPresent) => {}
		Err(error) => return Err(error),
	}
	Ok("vi".to_string())
}

/// Open a file in an editor.
pub fn open_file_in_editor(
	editor: String,
	filename: String,
) -> Result<process::ExitStatus, io::Error> {
	let mut cmd = process::Command::new(editor);
	let cmd = cmd.args([filename]);
	let mut child = cmd.spawn()?;
	child.wait()
}

#[cfg(test)]
mod tests {
	use super::*;
	use tempfile::tempdir;

	#[test]
	fn test_get_action() {
		assert_eq!(String::from("edit"), get_action().name);
	}

	#[test]
	fn test_editor() {
		env::set_var("EDITOR", "cat");
		assert!(editor().is_ok());
		assert_eq!("cat", editor().unwrap());
	}

	#[test]
	fn test_open_file_in_editor() {
		let dir = tempdir().unwrap();
		let test_filename = dir
			.path()
			.join("some-file.txt")
			.display()
			.to_string();

		// `cat FILE` with a non-existing file.
		let exitcode =
			open_file_in_editor(String::from("cat"), test_filename.clone())
				.unwrap();
		assert!(!exitcode.success());

		// `cat FILE` with an existing file.
		List::new().to_filename(test_filename.clone());
		let exitcode =
			open_file_in_editor(String::from("cat"), test_filename.clone())
				.unwrap();
		assert!(exitcode.success());
	}
}

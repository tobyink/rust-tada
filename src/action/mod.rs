use clap::{Arg, ArgMatches, Command};
use std::env;

/// Handy structure for holding subcommand info
pub struct Action {
	pub name: String,
	pub command: Command,
}

/// A type of file
pub enum FileType {
	TodoTxt,
	DoneTxt,
}

/// Utility functions for subcommands
impl Action {
	fn _add_todotxt_file_options(cmd: Command) -> Command {
		cmd.arg(
			Arg::new("file")
				.short('f')
				.long("file")
				.value_name("FILE")
				.help("the path to todo.txt"),
		)
	}

	/// Given a filetype and set of options, determines the exact file path.
	///
	/// Uses environment variables `TODO_FILE`, `TODO_DIR`, and `DONE_FILE`
	/// as fallbacks.
	fn determine_filename(filetype: FileType, args: &ArgMatches) -> String {
		// I do not like how deeply nested this function goes. I have
		// been trying to avoid more than four levels of indentation.
		// This function goes to eight!!!
		match filetype {
			FileType::TodoTxt => match args.get_one::<String>("file") {
				Some(f) => f.to_string(),
				None => match env::var("TODO_FILE") {
					Ok(file) => file,
					_ => match env::var("TODO_DIR") {
						Ok(dir) => dir + "/todo.txt",
						_ => match env::var("HOME") {
							Ok(dir) => dir + "/todo.txt",
							_ => {
								panic!("Could not determine path to todo.txt!")
							}
						},
					},
				},
			},
			FileType::DoneTxt => match args.get_one::<String>("donefile") {
				Some(f) => f.to_string(),
				None => match env::var("DONE_FILE") {
					Ok(file) => file,
					_ => match env::var("TODO_DIR") {
						Ok(dir) => dir + "/done.txt",
						_ => match env::var("HOME") {
							Ok(dir) => dir + "/done.txt",
							_ => {
								panic!("Could not determine path to done.txt!")
							}
						},
					},
				},
			},
		}
	}
}

pub mod show;

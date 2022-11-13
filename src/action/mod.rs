use crate::*;
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

	fn _add_output_options(cmd: Command) -> Command {
		cmd.arg(
			Arg::new("max-width")
				.long("max-width")
				.aliases(["maxwidth"])
				.value_parser(clap::value_parser!(usize))
				.value_name("COLS")
				.help("maximum width of terminal output"),
		)
		.arg(
			Arg::new("colour")
				.num_args(0)
				.long("colour")
				.aliases(["color"])
				.help("coloured output"),
		)
		.arg(
			Arg::new("no-colour")
				.num_args(0)
				.long("no-colour")
				.aliases(["no-color", "nocolour", "nocolor"])
				.help("plain output"),
		)
		.arg(
			Arg::new("show-created")
				.num_args(0)
				.long("show-created")
				.aliases(["showcreated", "created"])
				.help("show 'created' dates for tasks"),
		)
		.arg(
			Arg::new("show-finished")
				.num_args(0)
				.long("show-finished")
				.aliases(["showfinished", "finished"])
				.help("show 'finished' dates for tasks"),
		)
	}

	/// Given a set of options, turns them into an output config.
	fn build_output_config(args: &ArgMatches) -> ItemFormatConfig {
		let mut cfg = ItemFormatConfig::new();
		cfg.colour = if *args.get_one::<bool>("no-colour").unwrap() {
			false
		} else if *args.get_one::<bool>("colour").unwrap() {
			true
		} else {
			console::colors_enabled()
		};
		cfg.with_creation_date = *args.get_one::<bool>("show-created").unwrap();
		cfg.with_completion_date =
			*args.get_one::<bool>("show-finished").unwrap();
		cfg.width = *args
			.get_one::<usize>("max-width")
			.unwrap_or(&cfg.width);
		if cfg.width < 30 {
			panic!("max-width must be at least 30!");
		}
		cfg
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

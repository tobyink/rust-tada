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
				.help("the path or URL for todo.txt"),
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
		let mut cfg = ItemFormatConfig::new_based_on_terminal();
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
		if cfg.width < 36 {
			panic!("max-width must be at least 36!");
		}
		cfg
	}

	pub fn sort_items_by<'a>(
		sortby: &'a str,
		items: Vec<&'a Item>,
	) -> Vec<&'a Item> {
		let mut out = items.clone();
		match sortby.to_lowercase().as_str() {
			"urgency" | "urgent" | "urg" => {
				out.sort_by_cached_key(|i| i.urgency().unwrap_or(Urgency::Soon))
			}
			"importance" | "import" | "imp" => {
				out.sort_by_cached_key(|i| i.importance().unwrap_or('D'))
			}
			"size" | "tshirt" => out.sort_by_cached_key(|i| {
				i.tshirt_size().unwrap_or(TshirtSize::Medium)
			}),
			"alphabetical" | "alphabet" | "alpha" => {
				out.sort_by_cached_key(|i| i.description().to_lowercase())
			}
			"due-date" | "duedate" | "due" => {
				out.sort_by_cached_key(|i| i.due_date())
			}
			"original" | "orig" => (),
			"smart" => out.sort_by_cached_key(|i| i.smart_key()),
			_ => panic!("unknown sorting: '{}'", sortby),
		};
		out
	}

	/// Given a filetype and set of options, determines the exact file path.
	///
	/// Uses environment variables `TODO_FILE`, `TODO_DIR`, and `DONE_FILE`
	/// as fallbacks.
	fn determine_filename(filetype: FileType, args: &ArgMatches) -> String {
		match filetype {
			FileType::TodoTxt => Self::_determine_filename_for_todo_txt(args),
			FileType::DoneTxt => Self::_determine_filename_for_done_txt(args),
		}
	}

	fn _determine_filename_for_todo_txt(args: &ArgMatches) -> String {
		if let Some(f) = args.get_one::<String>("file") {
			return f.to_string();
		};
		if let Ok(f) = env::var("TODO_FILE") {
			return f;
		};
		let dir = env::var("TODO_DIR").unwrap_or_else(|_| {
			env::var("HOME").expect("Could not determine path to todo.txt!")
		});
		dir + "/todo.txt"
	}

	fn _determine_filename_for_done_txt(args: &ArgMatches) -> String {
		if let Some(f) = args.get_one::<String>("donefile") {
			return f.to_string();
		};
		if let Ok(f) = env::var("DONE_FILE") {
			return f;
		};
		let dir = env::var("TODO_DIR").unwrap_or_else(|_| {
			env::var("HOME").expect("Could not determine path to done.txt!")
		});
		dir + "/done.txt"
	}
}

pub mod add;
pub mod show;
pub mod urgent;

use crate::item::{Item, ItemFormatConfig};
use crate::list::{LineKind, List};
use clap::{Arg, ArgAction, ArgMatches, Command};
use promptly::prompt_default;
use std::{env, fs};

/// Handy structure for holding subcommand info
pub struct Action {
	pub name: String,
	pub command: Command,
}

/// A type of file
#[derive(Clone, Copy, Eq, PartialEq, PartialOrd, Ord)]
pub enum FileType {
	TodoTxt,
	DoneTxt,
	Config,
}

/// Whether the user has confirmed an action on an item.
#[derive(Clone, Copy, Eq, PartialEq, PartialOrd, Ord)]
pub enum ConfirmationStatus {
	Yes,
	No,
	Ask,
}

impl ConfirmationStatus {
	/// Initialize from ArgMatches.
	pub fn from_argmatches(args: &ArgMatches) -> Self {
		if *args.get_one::<bool>("no").unwrap() {
			Self::No
		} else if *args.get_one::<bool>("yes").unwrap() {
			Self::Yes
		} else {
			Self::Ask
		}
	}

	/// Possibly prompt a user for confirmation.
	pub fn check(
		&self,
		prompt_phrase: &str,
		yes_phrase: &str,
		no_phrase: &str,
	) -> bool {
		match self {
			ConfirmationStatus::Yes => {
				println!("{}\n", yes_phrase);
				true
			}
			ConfirmationStatus::No => {
				println!("{}\n", no_phrase);
				false
			}
			ConfirmationStatus::Ask => {
				let response = prompt_default(prompt_phrase, true).unwrap();
				if response {
					println!("{}\n", yes_phrase);
				} else {
					println!("{}\n", no_phrase);
				}
				response
			}
		}
	}
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
		.arg(
			Arg::new("local")
				.num_args(0)
				.short('l')
				.long("local")
				.help("look for files in local directory only"),
		)
	}

	fn _add_donetxt_file_options(cmd: Command) -> Command {
		cmd.arg(
			Arg::new("done-file")
				.long("done-file")
				.value_name("FILE")
				.help("the path or URL for done.txt"),
		)
	}

	fn _add_search_terms_option(cmd: Command) -> Command {
		cmd.arg(
			Arg::new("search-term")
				.action(ArgAction::Append)
				.required(true)
				.help("a tag, context, line number, or string"),
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
			Arg::new("show-lines")
				.num_args(0)
				.short('L')
				.long("show-lines")
				.aliases(["show-lines", "lines"])
				.help("show line numbers for tasks"),
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

	fn _add_prompt_options(cmd: Command) -> Command {
		cmd.arg(
			Arg::new("yes")
				.num_args(0)
				.short('y')
				.long("yes")
				.help("assume 'yes' to prompts"),
		)
		.arg(
			Arg::new("no")
				.num_args(0)
				.short('n')
				.long("no")
				.help("assume 'no' to prompts"),
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
		cfg.with_line_numbers = *args.get_one::<bool>("show-lines").unwrap();
		cfg.width = *args
			.get_one::<usize>("max-width")
			.unwrap_or(&cfg.width);
		if cfg.width < 48 {
			panic!("max-width must be at least 48!");
		}
		cfg
	}

	fn maybe_warnings(list: &List) {
		let mut done_blank = false;

		let count_finished = list
			.lines
			.iter()
			.filter(|l| {
				l.kind == LineKind::Item && l.item.clone().unwrap().completion()
			})
			.count();
		if count_finished > 9 {
			if !done_blank {
				println!();
				done_blank = true;
			}
			println!(
				"There are {} finished tasks. Consider running `tada archive`.",
				count_finished
			);
		}

		let count_blank = list
			.lines
			.iter()
			.filter(|l| l.kind != LineKind::Item)
			.count();
		if count_blank > 9 {
			if !done_blank {
				println!();
				// done_blank = true;
			}
			println!("There are {} blank/comment lines. Consider running `tada tidy`.", count_blank);
		}
	}

	/// Given a filetype and set of options, determines the exact file path.
	///
	/// Uses environment variables `TODO_FILE`, `TODO_DIR`, and `DONE_FILE`
	/// as fallbacks.
	pub fn determine_filename(filetype: FileType, args: &ArgMatches) -> String {
		match filetype {
			FileType::TodoTxt => Self::_determine_filename_for_todo_txt(args),
			FileType::DoneTxt => Self::_determine_filename_for_done_txt(args),
			FileType::Config => Self::_determine_filename_for_config(args),
		}
	}

	pub fn determine_search_terms(args: &ArgMatches) -> Vec<&String> {
		args.get_many::<String>("search-term").unwrap().collect()
	}

	pub fn _file_exists(path: &str) -> bool {
		match fs::metadata(path) {
			Ok(f) => f.is_file(),
			Err(_) => false,
		}
	}

	fn _determine_filename_for_todo_txt(args: &ArgMatches) -> String {
		let local_only = *args.get_one::<bool>("local").unwrap_or(&false);
		if local_only {
			let names =
				["todo.txt", "TODO", "TODO.TXT", "ToDo", "ToDo.txt", "todo"];
			for n in names {
				let qname = env::current_dir()
					.unwrap()
					.into_os_string()
					.into_string()
					.unwrap() + "/" + n;
				if Self::_file_exists(&qname) {
					return qname;
				}
			}
			panic!("Could not find a file called todo.txt or TODO in the current directory!")
		}

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
		let local_only = *args.get_one::<bool>("local").unwrap_or(&false);
		if local_only {
			let names =
				["done.txt", "DONE", "DONE.TXT", "Done", "Done.txt", "done"];
			for n in names {
				let qname = env::current_dir()
					.unwrap()
					.into_os_string()
					.into_string()
					.unwrap() + "/" + n;
				if Self::_file_exists(&qname) {
					return qname;
				}
			}
			panic!("Could not find a file called done.txt or DONE in the current directory!")
		}

		if let Some(f) = args.get_one::<String>("done-file") {
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

	fn _determine_filename_for_config(args: &ArgMatches) -> String {
		if let Some(f) = args.get_one::<String>("config-file") {
			return f.to_string();
		};
		if let Ok(f) = env::var("TADA_CONFIG") {
			return f;
		};
		let dir =
			env::var("HOME").expect("Could not determine path to tada.toml!");
		dir + "~/.config/tada.toml"
	}

	/// Given an item and a list of terms from the command line, checks whether the item matches at least one term.
	pub fn item_matches_terms(item: &Item, terms: &Vec<&String>) -> bool {
		for term in terms {
			match term.chars().next() {
				Some('@') => {
					if item.has_context(term) {
						return true;
					}
				}
				Some('+') => {
					if item.has_tag(term) {
						return true;
					}
				}
				Some('#') => {
					let n: usize = term.get(1..).unwrap().parse().unwrap();
					if item.line_number() == n {
						return true;
					}
				}
				_ => {
					let lc_term = term.to_lowercase();
					if item
						.description()
						.to_lowercase()
						.contains(&lc_term)
					{
						return true;
					}
				}
			}
		}
		false
	}
}

pub mod add;
pub mod archive;
pub mod done;
pub mod edit;
pub mod find;
pub mod important;
pub mod pull;
pub mod quick;
pub mod remove;
pub mod show;
pub mod tidy;
pub mod urgent;
pub mod zen;

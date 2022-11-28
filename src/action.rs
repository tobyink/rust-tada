use crate::item::{Item, TshirtSize, Urgency};
use crate::list::{LineKind, List};
use clap::{Arg, ArgAction, ArgMatches, Command};
use console::Style;
use promptly::prompt_default;
use std::{env, fs, io};
use substring::Substring;

/// Handy structure for holding subcommand metadata.
pub struct Action {
	pub name: String,
	pub command: Command,
}

/// A type of file.
#[derive(Clone, Copy, Eq, PartialEq, PartialOrd, Ord)]
pub enum FileType {
	TodoTxt,
	DoneTxt,
}

impl FileType {
	/// Given a set of options, determines the exact file path.
	///
	/// Uses environment variables `TODO_FILE`, `TODO_DIR`, and `DONE_FILE`
	/// as fallbacks.
	pub fn filename(&self, args: &ArgMatches) -> String {
		match self {
			Self::TodoTxt => Self::_filename_for_todotxt(args),
			Self::DoneTxt => Self::_filename_for_donetxt(args),
		}
	}

	/// Human-readable label for the file type.
	pub fn label(&self) -> String {
		match self {
			Self::TodoTxt => String::from("todo list"),
			Self::DoneTxt => String::from("done list"),
		}
	}

	/// Shortcut to determine the file path and load it as a List.
	pub fn load(&self, args: &ArgMatches) -> List {
		let filename = self.filename(args);
		let label = self.label();
		List::from_url(filename)
			.unwrap_or_else(|_| panic!("Could not read {}", label))
	}

	fn _file_exists(path: &str) -> bool {
		match fs::metadata(path) {
			Ok(f) => f.is_file(),
			Err(_) => false,
		}
	}

	fn _filename_for_todotxt(args: &ArgMatches) -> String {
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

	fn _filename_for_donetxt(args: &ArgMatches) -> String {
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

	/// Add some args to a Command so that it will expect a file of this type.
	pub fn add_args(&self, cmd: Command) -> Command {
		match self {
			Self::TodoTxt => Self::_add_args_for_todotxt(cmd),
			Self::DoneTxt => Self::_add_args_for_donetxt(cmd),
		}
	}

	fn _add_args_for_todotxt(cmd: Command) -> Command {
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

	fn _add_args_for_donetxt(cmd: Command) -> Command {
		cmd.arg(
			Arg::new("done-file")
				.long("done-file")
				.value_name("FILE")
				.help("the path or URL for done.txt"),
		)
	}
}

/// Provides pretty output for Item objects.
pub struct Outputter {
	pub width: usize,
	pub colour: bool,
	pub with_creation_date: bool,
	pub with_completion_date: bool,
	pub with_line_numbers: bool,
	pub with_newline: bool,
	pub line_number_digits: usize,
	pub io: Box<dyn io::Write>,
}

impl Outputter {
	/// Constructor for item format config, given an output width
	pub fn new(width: usize) -> Self {
		Self {
			width,
			colour: false,
			with_creation_date: false,
			with_completion_date: false,
			with_line_numbers: false,
			with_newline: true,
			line_number_digits: 2,
			io: Box::new(io::stdout()),
		}
	}

	/// Alternative constructor, which detects width from the terminal
	pub fn new_based_on_terminal() -> Self {
		let term = console::Term::stdout();
		let (_height, width) = term.size();
		Self::new(width.into())
	}

	/// Add some args to a Command so that it can instantiate a basic Outputter.
	pub fn add_args_minimal(cmd: Command) -> Command {
		cmd.arg(
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
	}

	/// Add some args to a Command so that it can instantiate a more complete Outputter.
	pub fn add_args(cmd: Command) -> Command {
		Self::add_args_minimal(cmd)
			.arg(
				Arg::new("max-width")
					.long("max-width")
					.aliases(["maxwidth"])
					.value_parser(clap::value_parser!(usize))
					.value_name("COLS")
					.help("maximum width of terminal output"),
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

	/// Initialize from minimal ArgMatches.
	pub fn from_argmatches_minimal(args: &ArgMatches) -> Self {
		let mut cfg = Self::new_based_on_terminal();
		cfg.colour = if *args.get_one::<bool>("no-colour").unwrap() {
			false
		} else if *args.get_one::<bool>("colour").unwrap() {
			true
		} else {
			console::colors_enabled()
		};
		cfg
	}

	/// Initialize from ArgMatches.
	pub fn from_argmatches(args: &ArgMatches) -> Self {
		let mut cfg = Self::from_argmatches_minimal(args);
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

	/// Write a heading row.
	pub fn write_heading(&mut self, heading: String) {
		let stream = &mut self.io;
		let mut hh: String = format!("# {}", heading);
		if self.colour {
			let s = Style::new()
				.white()
				.bright()
				.bold()
				.force_styling(true);
			hh = s.apply_to(hh).to_string();
		}
		if self.with_newline {
			writeln!(stream, "{}", hh).expect("panik");
		} else {
			write!(stream, "{}", hh).expect("panik");
		}
	}

	/// Write a separator row.
	pub fn write_separator(&mut self) {
		let stream = &mut self.io;
		writeln!(stream).expect("panik");
	}

	/// Write a status line.
	pub fn write_status(&mut self, status: String) {
		let stream = &mut self.io;
		let mut hh: String = status;
		if self.colour {
			let s = Style::new()
				.white()
				.bright()
				.force_styling(true);
			hh = s.apply_to(hh).to_string();
		}
		if self.with_newline {
			writeln!(stream, "{}", hh).expect("panik");
		} else {
			write!(stream, "{}", hh).expect("panik");
		}
	}

	/// Write a hint line.
	pub fn write_notice(&mut self, hint: String) {
		let stream = &mut self.io;
		let mut hh: String = hint;
		if self.colour {
			let s = Style::new().magenta().force_styling(true);
			hh = s.apply_to(hh).to_string();
		}
		if self.with_newline {
			writeln!(stream, "{}", hh).expect("panik");
		} else {
			write!(stream, "{}", hh).expect("panik");
		}
	}

	/// Write a error line.
	pub fn write_error(&mut self, errstr: String) {
		let stream = &mut self.io;
		let mut hh: String = errstr;
		if self.colour {
			let s = Style::new().red().force_styling(true);
			hh = s.apply_to(hh).to_string();
		}
		if self.with_newline {
			writeln!(stream, "{}", hh).expect("panik");
		} else {
			write!(stream, "{}", hh).expect("panik");
		}
	}

	/// Write an item. (Not in todo.txt format!)
	///
	/// Allows for pretty formatting, etc.
	pub fn write_item(&mut self, i: &Item) {
		let stream = &mut self.io;
		let mut r: String = String::new();

		if i.completion() {
			r.push_str("x ");
		} else {
			r.push_str("  ");
		}

		if i.priority() == '\0' {
			r.push_str("(?) ");
		} else {
			let style = match i.importance() {
				Some('A') => Style::new().red().bold().force_styling(true),
				Some('B') => Style::new().yellow().bold().force_styling(true),
				Some('C') => Style::new().green().bold().force_styling(true),
				Some(_) => Style::new().bold().force_styling(true),
				_ => Style::new(),
			};
			let paren = format!("({}) ", style.apply_to(i.priority()));
			r.push_str(&paren);
		}

		if self.with_completion_date {
			if i.completion() && i.completion_date().is_some() {
				let date = i
					.completion_date()
					.unwrap()
					.format("%Y-%m-%d ")
					.to_string();
				r.push_str(&date);
			} else if i.completion() {
				r.push_str("????-??-?? ");
			} else {
				r.push_str("           ");
			}
		}

		if self.with_creation_date {
			if i.creation_date().is_some() {
				let date = i
					.creation_date()
					.unwrap()
					.format("%Y-%m-%d ")
					.to_string();
				r.push_str(&date);
			} else {
				r.push_str("????-??-?? ");
			}
		}

		if self.with_line_numbers {
			r.push_str(
				format!(
					"#{:0width$} ",
					i.line_number(),
					width = self.line_number_digits
				)
				.as_str(),
			)
		}

		let len = self.width - console::strip_ansi_codes(&r).len();
		r.push_str(i.description().substring(0, len));

		if i.completion() || !i.is_startable() {
			if self.colour {
				r = format!(
					"{}",
					Style::new()
						.dim()
						.force_styling(true)
						.apply_to(console::strip_ansi_codes(&r).to_string())
				);
			} else {
				r = console::strip_ansi_codes(&r).to_string();
			}
		} else if !self.colour {
			r = console::strip_ansi_codes(&r).to_string();
		}

		if self.with_newline {
			writeln!(stream, "{}", r).expect("panik");
		} else {
			write!(stream, "{}", r).expect("panik");
		}
	}
}

impl Default for Outputter {
	fn default() -> Self {
		Self::new_based_on_terminal()
	}
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
		outputter: &mut Outputter,
		prompt_phrase: &str,
		yes_phrase: &str,
		no_phrase: &str,
	) -> bool {
		match self {
			ConfirmationStatus::Yes => {
				outputter.write_notice(format!("{}\n", yes_phrase));
				true
			}
			ConfirmationStatus::No => {
				outputter.write_notice(format!("{}\n", no_phrase));
				false
			}
			ConfirmationStatus::Ask => {
				let response = prompt_default(prompt_phrase, true).unwrap();
				if response {
					outputter.write_notice(format!("{}\n", yes_phrase));
				} else {
					outputter.write_notice(format!("{}\n", no_phrase));
				}
				response
			}
		}
	}

	/// Add some args to a Command so that it can prompt for yes/no questions.
	pub fn add_args(cmd: Command) -> Command {
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
}

/// Structure for holding command-line search terms.
pub struct SearchTerms {
	pub terms: Vec<String>,
}

impl SearchTerms {
	/// Create a new empty set of search terms.
	pub fn new() -> Self {
		Self { terms: Vec::new() }
	}

	/// Create a new set of search terms from a Vec of Strings.
	pub fn from_vec(terms: Vec<String>) -> Self {
		Self { terms }
	}

	/// Create a new set of search terms from a single String.
	pub fn from_string(term: &str) -> Self {
		Self {
			terms: Vec::from([String::from(term)]),
		}
	}

	/// Add some args to a Command so that it can accept search terms.
	pub fn add_args(cmd: Command) -> Command {
		cmd.arg(
			Arg::new("search-term")
				.action(ArgAction::Append)
				.required(true)
				.help("a tag, context, line number, or string"),
		)
	}

	/// Read search terms from ArgMatches.
	pub fn from_argmatches(args: &ArgMatches) -> Self {
		let terms = args
			.get_many::<String>("search-term")
			.unwrap()
			.cloned()
			.collect();
		Self { terms }
	}

	/// Given an item, checks whether the item matches at least one term.
	pub fn item_matches(&self, item: &Item) -> bool {
		for term in &self.terms {
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

impl Default for SearchTerms {
	fn default() -> Self {
		Self::new()
	}
}

/// An order for sorting items into.
pub enum SortOrder {
	Urgency,
	Importance,
	TshirtSize,
	Alphabetical,
	DueDate,
	Original,
	Smart,
}

/// An error raised when given an unknown sort order.
#[derive(Debug, Clone)]
pub struct InvalidSortOrder;

impl SortOrder {
	/// Add some args to a Command so that it can accept a sort order.
	pub fn add_args(cmd: Command, default_val: &str) -> Command {
		cmd.arg(
			Arg::new("sort")
				.num_args(1)
				.short('s')
				.long("sort")
				.value_name("BY")
				.help(format!(
					"sort by 'smart', 'urgency', 'importance', 'size', 'alpha', or 'due' (default: {})",
					default_val
				))
		)
	}

	/// Read sort order from ArgMatches.
	pub fn from_argmatches(args: &ArgMatches, default_val: &str) -> Self {
		let default_string = String::from(default_val);
		let from_user = args
			.get_one::<String>("sort")
			.unwrap_or(&default_string);
		Self::from_string(from_user)
			.unwrap_or_else(|_| Self::from_string(default_val).unwrap())
	}

	/// Accept string sort orders like "urgency" and return a SortOrder.
	pub fn from_string(sortby: &str) -> Result<Self, InvalidSortOrder> {
		match sortby.to_lowercase().as_str() {
			"urgency" | "urgent" | "urg" => Ok(SortOrder::Urgency),
			"importance" | "import" | "imp" | "important" => {
				Ok(SortOrder::Importance)
			}
			"tshirtsize" | "size" | "tshirt" | "quick" => {
				Ok(SortOrder::TshirtSize)
			}
			"alphabetical" | "alphabet" | "alpha" => {
				Ok(SortOrder::Alphabetical)
			}
			"due-date" | "duedate" | "due" => Ok(SortOrder::DueDate),
			"original" | "orig" => Ok(SortOrder::Original),
			"smart" => Ok(SortOrder::Smart),
			_ => Err(InvalidSortOrder),
		}
	}

	/// Sort items by this sort order.
	pub fn sort_items<'a>(&self, items: Vec<&'a Item>) -> Vec<&'a Item> {
		let mut out = items.clone();
		match self {
			SortOrder::Urgency => {
				out.sort_by_cached_key(|i| i.urgency().unwrap_or(Urgency::Soon))
			}
			SortOrder::Importance => {
				out.sort_by_cached_key(|i| i.importance().unwrap_or('D'))
			}
			SortOrder::TshirtSize => out.sort_by_cached_key(|i| {
				i.tshirt_size().unwrap_or(TshirtSize::Medium)
			}),
			SortOrder::Alphabetical => {
				out.sort_by_cached_key(|i| i.description().to_lowercase())
			}
			SortOrder::DueDate => out.sort_by_cached_key(|i| i.due_date()),
			SortOrder::Original => out.sort_by_cached_key(|i| i.line_number()),
			SortOrder::Smart => out.sort_by_cached_key(|i| i.smart_key()),
		};
		out
	}
}

fn maybe_housekeeping_warnings(outputter: &mut Outputter, list: &List) {
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
			outputter.write_separator();
			done_blank = true;
		}
		outputter.write_notice(format!(
			"There are {} finished tasks. Consider running `tada archive`.",
			count_finished
		));
	}

	let count_blank = list
		.lines
		.iter()
		.filter(|l| l.kind != LineKind::Item)
		.count();
	if count_blank > 9 {
		if !done_blank {
			outputter.write_separator();
			// done_blank = true;
		}
		outputter.write_notice(format!(
			"There are {} blank/comment lines. Consider running `tada tidy`.",
			count_blank
		));
	}
}

pub mod add;
pub mod archive;
pub mod done;
pub mod edit;
pub mod find;
pub mod important;
pub mod path;
pub mod pull;
pub mod quick;
pub mod remove;
pub mod show;
pub mod tidy;
pub mod urgent;
pub mod zen;

//! Add a task to the todo list

use crate::action::*;
use crate::item::{Item, Urgency};
use crate::list::{Line, List};
use clap::{Arg, ArgMatches, Command};

/// Options for the `add` subcommand.
pub fn get_action() -> Action {
	let name = String::from("add");
	let mut command = Command::new("add")
		.about("Add a task to the todo list")
		.after_help("After success, displays the added task.")
		.arg(Arg::new("task").help("Task text (may use todo.txt features)"));

	command = FileType::TodoTxt.add_args(command);
	command = AddActionConfig::add_args(command);

	Action { name, command }
}

/// Config object for the `add` action.
pub struct AddActionConfig {
	pub no_date: bool,
	pub no_fixup: bool,
	pub urgency: Option<Urgency>,
	pub quiet: bool,
	pub outputter: Outputter,
}

impl Default for AddActionConfig {
	fn default() -> Self {
		Self::new()
	}
}

impl AddActionConfig {
	/// Constructor.
	pub fn new() -> Self {
		Self {
			no_date: false,
			no_fixup: false,
			urgency: None,
			quiet: false,
			outputter: Outputter::default(),
		}
	}

	/// Add arguments to a clap Command for the `add` action's options.
	pub fn add_args(cmd: Command) -> Command {
		let cmd = cmd
			.arg(
				Arg::new("no-date")
					.num_args(0)
					.long("no-date")
					.aliases(["nodate"])
					.help(
						"Don't automatically add a creation date to the task",
					),
			)
			.arg(
				Arg::new("no-fixup")
					.num_args(0)
					.long("no-fixup")
					.aliases(["nofixup"])
					.help("Don't try to fix task syntax"),
			)
			.arg(
				Arg::new("quiet")
					.num_args(0)
					.long("quiet")
					.help("Quieter output"),
			)
			.arg(
				Arg::new("today")
					.num_args(0)
					.short('T')
					.long("today")
					.help("Include a due date of today"),
			)
			.arg(
				Arg::new("soon")
					.num_args(0)
					.short('S')
					.long("soon")
					.aliases(["overmorrow"])
					.help("Include a due date of overmorrow"),
			)
			.arg(
				Arg::new("next-week")
					.num_args(0)
					.short('W')
					.long("next-week")
					.aliases(["nextweek"])
					.help("Include a due date the end of next week"),
			)
			.arg(
				Arg::new("next-month")
					.num_args(0)
					.short('M')
					.long("next-month")
					.aliases(["nextmonth"])
					.help("Include a due date the end of next month"),
			);
		Outputter::add_args(cmd)
	}

	/// Create an AddActionConfig from an appropriate ArgMatches.
	pub fn from_argmatches(args: &ArgMatches) -> Self {
		let no_date = *args.get_one::<bool>("no-date").unwrap();
		let no_fixup = *args.get_one::<bool>("no-fixup").unwrap();
		let urgency = if *args.get_one::<bool>("today").unwrap() {
			Some(Urgency::Today)
		} else if *args.get_one::<bool>("soon").unwrap() {
			Some(Urgency::Soon)
		} else if *args.get_one::<bool>("next-week").unwrap() {
			Some(Urgency::NextWeek)
		} else if *args.get_one::<bool>("next-month").unwrap() {
			Some(Urgency::NextMonth)
		} else {
			None
		};
		let quiet = *args.get_one::<bool>("quiet").unwrap();
		let outputter = Outputter::from_argmatches(args);
		Self {
			no_date,
			no_fixup,
			urgency,
			quiet,
			outputter,
		}
	}
}

/// Execute the `add` subcommand.
#[cfg(not(tarpaulin_include))]
pub fn execute(args: &ArgMatches) {
	let mut cfg = AddActionConfig::from_argmatches(args);
	let input = args.get_one::<String>("task").unwrap();
	let new_line = process_line(input, &cfg);

	if !cfg.quiet {
		cfg.outputter
			.write_item(new_line.item.as_ref().unwrap());
	}

	let filename = FileType::TodoTxt.filename(args);
	List::append_lines_to_url(filename, Vec::from([&new_line]));
}

/// Process a line to be added to a todo list.
pub fn process_line(input: &str, cfg: &AddActionConfig) -> Line {
	let mut item = Item::parse(input);

	if item.creation_date().is_none() && !cfg.no_date {
		item.set_creation_date(chrono::Utc::now().date_naive());
	}

	if let Some(u) = cfg.urgency {
		item.set_urgency(u);
	}

	if !cfg.no_fixup {
		item = item.fixup(!cfg.quiet);
	}

	Line::from_item(item)
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::list::LineKind;

	#[test]
	fn test_get_action() {
		assert_eq!(String::from("add"), get_action().name);
	}

	#[test]
	fn test_add_action_config() {
		let cfg = AddActionConfig::new();
		assert_eq!(false, cfg.no_date);
		assert_eq!(false, cfg.no_fixup);
		assert_eq!(None, cfg.urgency);
		assert_eq!(false, cfg.quiet);

		let cfg = AddActionConfig::default();
		assert_eq!(false, cfg.no_date);
		assert_eq!(false, cfg.no_fixup);
		assert_eq!(None, cfg.urgency);
		assert_eq!(false, cfg.quiet);

		let matches = get_action()
			.command
			.get_matches_from(vec!["add"]);
		let cfg = AddActionConfig::from_argmatches(&matches);
		assert_eq!(false, cfg.no_date);
		assert_eq!(false, cfg.no_fixup);
		assert_eq!(None, cfg.urgency);
		assert_eq!(false, cfg.quiet);

		let matches = get_action().command.get_matches_from(vec![
			"add",
			"-T",
			"--no-date",
		]);
		let cfg = AddActionConfig::from_argmatches(&matches);
		assert_eq!(true, cfg.no_date);
		assert_eq!(false, cfg.no_fixup);
		assert_eq!(Some(Urgency::Today), cfg.urgency);
		assert_eq!(false, cfg.quiet);

		let matches = get_action().command.get_matches_from(vec![
			"add",
			"-S",
			"--no-fixup",
		]);
		let cfg = AddActionConfig::from_argmatches(&matches);
		assert_eq!(false, cfg.no_date);
		assert_eq!(true, cfg.no_fixup);
		assert_eq!(Some(Urgency::Soon), cfg.urgency);
		assert_eq!(false, cfg.quiet);

		let matches = get_action()
			.command
			.get_matches_from(vec!["add", "-W"]);
		let cfg = AddActionConfig::from_argmatches(&matches);
		assert_eq!(false, cfg.no_date);
		assert_eq!(false, cfg.no_fixup);
		assert_eq!(Some(Urgency::NextWeek), cfg.urgency);
		assert_eq!(false, cfg.quiet);

		let matches = get_action().command.get_matches_from(vec![
			"add",
			"--no-fixup",
			"-M",
			"--no-date",
			"--quiet",
		]);
		let cfg = AddActionConfig::from_argmatches(&matches);
		assert_eq!(true, cfg.no_date);
		assert_eq!(true, cfg.no_fixup);
		assert_eq!(Some(Urgency::NextMonth), cfg.urgency);
		assert_eq!(true, cfg.quiet);
	}

	#[test]
	pub fn test_process_line() {
		let cfg = AddActionConfig {
			no_date: true,
			no_fixup: true,
			urgency: None,
			quiet: true,
			outputter: Outputter::default(),
		};
		let line = process_line(&String::from("ABC start:today"), &cfg);
		assert_eq!(LineKind::Item, line.kind);
		let item = line.item.unwrap();
		assert_eq!("ABC start:today", item.description());
		assert_eq!(None, item.creation_date());
		assert_eq!("today", item.kv().get("start").unwrap());

		let cfg = AddActionConfig {
			no_date: false,
			no_fixup: false,
			urgency: Some(Urgency::Today),
			quiet: true,
			outputter: Outputter::default(),
		};
		let line = process_line(&String::from("ABC start:today"), &cfg);
		assert_eq!(LineKind::Item, line.kind);
		let item = line.item.unwrap();
		assert!(item.creation_date().is_some());
		assert_eq!(item.creation_date(), item.start_date());
		assert_eq!(item.creation_date(), item.due_date());
		assert_ne!("today", item.kv().get("start").unwrap());
	}
}

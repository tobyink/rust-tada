use crate::action::{Action, FileType};
use crate::item::{Item, Urgency};
use crate::list::{Line, List};
use clap::{Arg, ArgMatches, Command};
use std::io;

/// Options for the `add` subcommand.
pub fn get_action() -> Action {
	let name = String::from("add");
	let mut command = Command::new("add")
		.about("Add a task to the todo list")
		.after_help("After success, displays the added task.")
		.arg(Arg::new("task").help("Task text (may use todo.txt features)"));

	command = Action::_add_todotxt_file_options(command);
	command = command
		.arg(
			Arg::new("no-date")
				.num_args(0)
				.long("no-date")
				.aliases(["nodate"])
				.help("Don't automatically add a creation date to the task"),
		)
		.arg(
			Arg::new("no-fixup")
				.num_args(0)
				.long("no-fixup")
				.aliases(["nofixup"])
				.help("Don't try to fix task syntax"),
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
	command = Action::_add_output_options(command);

	Action { name, command }
}

/// Execute the `add` subcommand.
pub fn execute(args: &ArgMatches) {
	let input = args.get_one::<String>("task").unwrap();
	let mut item = Item::parse(input);

	let no_date = *args.get_one::<bool>("no-date").unwrap();
	if item.creation_date().is_none() && !no_date {
		item.set_creation_date(chrono::Utc::now().date_naive());
	}

	if *args.get_one::<bool>("today").unwrap() {
		item.set_urgency(Urgency::Today);
	} else if *args.get_one::<bool>("soon").unwrap() {
		item.set_urgency(Urgency::Soon);
	} else if *args.get_one::<bool>("next-week").unwrap() {
		item.set_urgency(Urgency::NextWeek);
	} else if *args.get_one::<bool>("next-month").unwrap() {
		item.set_urgency(Urgency::NextMonth);
	}

	let no_fixup = *args.get_one::<bool>("no-fixup").unwrap();
	if !no_fixup {
		item = item.fixup(true);
	}

	let line = Line::from_item(item);
	let filename = Action::determine_filename(FileType::TodoTxt, args);
	List::append_lines_to_url(filename, Vec::from([&line]));

	let cfg = Action::build_output_config(args);
	let mut out = io::stdout();
	line.item.unwrap().write_to(&mut out, &cfg);
}

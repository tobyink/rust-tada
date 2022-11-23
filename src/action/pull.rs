use crate::action::{Action, FileType};
use crate::item::{Item, ItemFormatConfig, Urgency};
use crate::list::{LineKind, List};
use clap::{Arg, ArgAction, ArgMatches, Command};
use promptly::prompt_default;
use std::io;

/// Options for the `pull` subcommand.
pub fn get_action() -> Action {
	let name = String::from("pull");
	let mut command = Command::new("pull")
		.about("Reschedule a task or tasks to be done today (or another date)")
		.after_help("If a task has a start date, that will be set to today.");

	command = Action::_add_todotxt_file_options(command);
	command = Action::_add_output_options(command);
	command = command
		.arg(
			Arg::new("search-term")
				.action(ArgAction::Append)
				.required(true)
				.help("a tag, context, line number, or string"),
		)
		.arg(
			Arg::new("today")
				.num_args(0)
				.short('T')
				.long("today")
				.help("Set a due date of today (default)"),
		)
		.arg(
			Arg::new("soon")
				.num_args(0)
				.short('S')
				.long("soon")
				.aliases(["overmorrow"])
				.help("Set a due date of overmorrow"),
		)
		.arg(
			Arg::new("next-week")
				.num_args(0)
				.short('W')
				.long("next-week")
				.aliases(["nextweek"])
				.help("Set a due date the end of next week"),
		)
		.arg(
			Arg::new("next-month")
				.num_args(0)
				.short('M')
				.long("next-month")
				.aliases(["nextmonth"])
				.help("Set a due date the end of next month"),
		)
		.arg(
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
		);

	Action { name, command }
}

/// Execute the `pull` subcommand.
pub fn execute(args: &ArgMatches) {
	let todo_filename = Action::determine_filename(FileType::TodoTxt, args);
	let list = List::from_url(todo_filename.clone())
		.expect("Could not read todo list");
	let mut cfg = Action::build_output_config(args);
	cfg.line_number_digits = list.lines.len().to_string().len();

	let terms: Vec<&String> = args
		.get_many::<String>("search-term")
		.unwrap()
		.collect();
	let mut new_list = List::new();

	let opt = if *args.get_one::<bool>("no").unwrap() {
		'N'
	} else if *args.get_one::<bool>("yes").unwrap() {
		'Y'
	} else {
		'?'
	};

	let urgency = if *args.get_one::<bool>("today").unwrap() {
		Urgency::Today
	} else if *args.get_one::<bool>("soon").unwrap() {
		Urgency::Soon
	} else if *args.get_one::<bool>("next-week").unwrap() {
		Urgency::NextWeek
	} else if *args.get_one::<bool>("next-month").unwrap() {
		Urgency::NextMonth
	} else {
		Urgency::Today
	};

	let mut count = 0;
	for line in list.lines {
		match line.kind {
			LineKind::Item => {
				let item = line.item.clone().unwrap();
				if Action::item_matches_terms(&item, &terms)
					&& (!item.completion())
					&& check_if_pull(&item, opt, &mut io::stdout(), &cfg)
				{
					count += 1;
					new_list.lines.push(line.but_pull(urgency));
				} else {
					new_list.lines.push(line);
				}
			}
			_ => new_list.lines.push(line),
		}
	}

	if count > 0 {
		new_list.to_url(todo_filename);
	}

	Action::maybe_warnings(&new_list);
}

/// Asks whether to pull an item, and prints out the response before returning a bool.
pub fn check_if_pull(
	item: &Item,
	opt: char,
	out: &mut std::io::Stdout,
	cfg: &ItemFormatConfig,
) -> bool {
	item.write_to(out, cfg);

	if opt == 'Y' {
		println!("Rescheduling\n");
		return true;
	}

	if opt == 'N' {
		println!("Skipping\n");
		return false;
	}

	let response = prompt_default("Reschedule?", true).unwrap();
	if response {
		println!("Rescheduling\n");
		return true;
	}

	println!("Skipping\n");
	false
}

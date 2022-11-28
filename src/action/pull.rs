//! Reschedule a task or tasks to be done today (or another date)

use crate::action::*;
use crate::item::{Item, Urgency};
use crate::list::{LineKind, List};
use clap::{Arg, ArgMatches, Command};

/// Options for the `pull` subcommand.
pub fn get_action() -> Action {
	let name = String::from("pull");
	let mut command = Command::new("pull")
		.about("Reschedule a task or tasks to be done today (or another date)")
		.after_help("If a task has a start date, that will be set to today.");

	command = FileType::TodoTxt.add_args(command);
	command = Outputter::add_args(command);
	command = SearchTerms::add_args(command);
	command = command
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
		);
	command = ConfirmationStatus::add_args(command);

	Action { name, command }
}

/// Execute the `pull` subcommand.
pub fn execute(args: &ArgMatches) {
	let todo_filename = FileType::TodoTxt.filename(args);
	let list = List::from_url(todo_filename.clone())
		.expect("Could not read todo list");

	let mut outputter = Outputter::from_argmatches(args);
	outputter.line_number_digits = list.lines.len().to_string().len();

	let search_terms = SearchTerms::from_argmatches(args);
	let mut new_list = List::new();

	let confirmation = ConfirmationStatus::from_argmatches(args);
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
				if search_terms.item_matches(&item)
					&& (!item.completion())
					&& check_if_pull(&item, &mut outputter, confirmation)
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

	maybe_housekeeping_warnings(&mut outputter, &new_list);
}

/// Asks whether to pull an item, and prints out the response before returning a bool.
pub fn check_if_pull(
	item: &Item,
	outputter: &mut Outputter,
	status: ConfirmationStatus,
) -> bool {
	outputter.write_item(item);
	status.check(outputter, "Reschedule?", "Rescheduling", "Skipping")
}

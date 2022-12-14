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

	let (new_list, count) = pull_items_forward_in_list(
		list,
		search_terms,
		urgency,
		confirmation,
		&mut outputter,
	);
	if count > 0 {
		new_list.to_url(todo_filename);
	}

	maybe_housekeeping_warnings(&mut outputter, &new_list);
}

/// Given a list, set of search terms, and an urgency, creates a copy of the list
/// with all items matching the search terms "pulled forward" to have that urgency.
///
/// The confirmation status and outputter will be used to check whether each
/// individual item should be altered.
///
/// Also returns the number of items changed.
pub fn pull_items_forward_in_list(
	list: List,
	search_terms: SearchTerms,
	urgency: Urgency,
	confirmation: ConfirmationStatus,
	outputter: &mut Outputter,
) -> (List, usize) {
	let mut new_list = List::new();
	let mut count = 0;
	for line in list.lines {
		match line.kind {
			LineKind::Item => {
				let item = line.item.clone().unwrap();
				if search_terms.item_matches(&item)
					&& (!item.completion())
					&& check_if_pull(&item, outputter, confirmation)
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
	(new_list, count)
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

#[cfg(test)]
mod tests {
	use super::*;
	use crate::Line;
	use chrono::{Duration, Utc};
	use tempfile::tempdir;

	#[test]
	fn test_get_action() {
		assert_eq!(String::from("pull"), get_action().name);
	}

	#[test]
	fn test_check_if_pull() {
		let dir = tempdir().unwrap();
		let buffer_filename = dir
			.path()
			.join("buffer.txt")
			.display()
			.to_string();
		let mut i = Item::new();
		i.set_description(String::from("XYZ"));

		let mut o = Outputter::new(9999);
		o.colour = false;
		o.io = Box::new(fs::File::create(buffer_filename.clone()).unwrap());
		let r = check_if_pull(&i, &mut o, ConfirmationStatus::Yes);
		assert_eq!(true, r);
		let got_output = fs::read_to_string(buffer_filename.clone()).unwrap();
		assert_eq!(String::from("  (?) XYZ\nRescheduling\n\n"), got_output);

		let mut o = Outputter::new(9999);
		o.colour = false;
		o.io = Box::new(fs::File::create(buffer_filename.clone()).unwrap());
		let r = check_if_pull(&i, &mut o, ConfirmationStatus::No);
		assert_eq!(false, r);
		let got_output = fs::read_to_string(buffer_filename.clone()).unwrap();
		assert_eq!(String::from("  (?) XYZ\nSkipping\n\n"), got_output);
	}

	#[test]
	fn test_pull_items_forward_in_list() {
		let source_list = List {
			lines: Vec::from([
				Line::from_string(String::from("Foo1 start:3999-01-01"), 0),
				Line::from_string(String::from("Foo2 due:3999-01-01"), 0),
				Line::from_string(String::from(""), 0),
				Line::from_string(String::from("Bar"), 0),
			]),
			path: None,
		};

		let (got, count) = pull_items_forward_in_list(
			source_list,
			SearchTerms {
				terms: Vec::from([String::from("foo")]),
			},
			Urgency::Soon,
			ConfirmationStatus::Yes,
			&mut Outputter::new(1000),
		);
		assert_eq!(2, count);

		let got_items = got.items();

		let item = got_items.get(0).unwrap();
		assert_eq!(Some(Utc::now().date_naive()), item.start_date());
		assert_eq!(
			Some(Utc::now().date_naive() + Duration::days(2)),
			item.due_date()
		);

		let item = got_items.get(1).unwrap();
		assert_eq!(None, item.start_date());
		assert_eq!(
			Some(Utc::now().date_naive() + Duration::days(2)),
			item.due_date()
		);

		let item = got_items.get(2).unwrap();
		assert_eq!(None, item.start_date());
		assert_eq!(None, item.due_date());
	}
}

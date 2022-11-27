use crate::action::*;
use crate::item::Item;
use crate::list::{LineKind, List};
use clap::{Arg, ArgMatches, Command};

/// Options for the `done` subcommand.
pub fn get_action() -> Action {
	let name = String::from("done");
	let mut command =
		Command::new("done").about("Mark a task or tasks as done");

	command = FileType::TodoTxt.add_args(command);
	command = Outputter::add_args(command);
	command = SearchTerms::add_args(command);
	command = command.arg(
		Arg::new("no-date")
			.num_args(0)
			.long("no-date")
			.aliases(["nodate"])
			.help("Don't automatically add a completion date to the task"),
	);
	command = ConfirmationStatus::add_args(command);

	Action { name, command }
}

/// Execute the `done` subcommand.
#[cfg(not(tarpaulin_include))]
pub fn execute(args: &ArgMatches) {
	let todo_filename = FileType::TodoTxt.filename(args);
	let list = List::from_url(todo_filename.clone())
		.expect("Could not read todo list");
	let search_terms = SearchTerms::from_argmatches(args);
	let mut outputter = Outputter::from_argmatches(args);
	outputter.line_number_digits = list.lines.len().to_string().len();
	let confirmation = ConfirmationStatus::from_argmatches(args);
	let include_date = !*args.get_one::<bool>("no-date").unwrap();

	let (count, new_list) = mark_items_done_in_list(
		list,
		search_terms,
		&mut outputter,
		confirmation,
		include_date,
	);

	if count > 0 {
		new_list.to_url(todo_filename);
		outputter.write_status(format!("Marked {} tasks complete!", count));
	} else {
		outputter.write_status(String::from("No actions taken."));
	}

	maybe_housekeeping_warnings(&mut outputter, &new_list);
}

/// Return a new list with certain tasks in the given list marked as complete, based on the
/// search terms. Also returns a count of items modified.
pub fn mark_items_done_in_list(
	input: List,
	search_terms: SearchTerms,
	outputter: &mut Outputter,
	status: ConfirmationStatus,
	include_date: bool,
) -> (usize, List) {
	let mut new_list = List::new();
	let mut count: usize = 0;

	for line in input.lines {
		match line.kind {
			LineKind::Item => {
				let item = line.item.clone().unwrap();
				if search_terms.item_matches(&item)
					&& (!item.completion())
					&& check_if_complete(&item, outputter, status)
				{
					count += 1;
					new_list.lines.push(line.but_done(include_date));
				} else {
					new_list.lines.push(line);
				}
			}
			_ => new_list.lines.push(line),
		}
	}

	(count, new_list)
}

/// Asks whether to mark an item as complete, and prints out the response before returning a bool.
pub fn check_if_complete(
	item: &Item,
	outputter: &mut Outputter,
	status: ConfirmationStatus,
) -> bool {
	outputter.write_item(item);
	status.check(outputter, "Mark finished?", "Marking finished", "Skipping")
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::list::Line;
	use chrono::Utc;
	use tempfile::tempdir;

	#[test]
	fn test_get_action() {
		assert_eq!(String::from("done"), get_action().name);
	}

	#[test]
	fn test_check_if_complete() {
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
		let r = check_if_complete(&i, &mut o, ConfirmationStatus::Yes);
		assert_eq!(true, r);
		let got_output = fs::read_to_string(buffer_filename.clone()).unwrap();
		assert_eq!(String::from("  (?) XYZ\nMarking finished\n\n"), got_output);

		let mut o = Outputter::new(9999);
		o.colour = false;
		o.io = Box::new(fs::File::create(buffer_filename.clone()).unwrap());
		let r = check_if_complete(&i, &mut o, ConfirmationStatus::No);
		assert_eq!(false, r);
		let got_output = fs::read_to_string(buffer_filename.clone()).unwrap();
		assert_eq!(String::from("  (?) XYZ\nSkipping\n\n"), got_output);
	}

	#[test]
	fn test_mark_items_done_in_list() {
		let lines: Vec<Line> = Vec::from([
			Line::from_string(String::from("x 2000-01-01 Foo"), 0),
			Line::from_string(String::from("2000-01-02 Foo"), 0),
			Line::from_string(String::from("# Foo"), 0),
			Line::from_string(String::from("Bar"), 0),
		]);
		let mut o = Outputter::new(9999);
		o.colour = false;
		o.io = Box::new(Vec::<u8>::new());

		let mut initial_list = List::new();
		initial_list.lines = lines.clone();
		let (count, got) = mark_items_done_in_list(
			initial_list,
			SearchTerms {
				terms: vec![String::from("foo")],
			},
			&mut o,
			ConfirmationStatus::Yes,
			false,
		);

		assert_eq!(1, count);
		assert_eq!(
			"x 2000-01-01 Foo\n\
			x 2000-01-02 Foo\n\
			# Foo\n\
			Bar\n",
			got.serialize()
		);

		let mut initial_list = List::new();
		initial_list.lines = lines.clone();
		let (count, got) = mark_items_done_in_list(
			initial_list,
			SearchTerms {
				terms: vec![String::from("foo")],
			},
			&mut o,
			ConfirmationStatus::Yes,
			true,
		);

		assert_eq!(1, count);
		assert_eq!(
			format!(
				"x 2000-01-01 Foo\n\
				x {} 2000-01-02 Foo\n\
				# Foo\n\
				Bar\n",
				Utc::now().date_naive().format("%Y-%m-%d")
			),
			got.serialize()
		);
	}
}

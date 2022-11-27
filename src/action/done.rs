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
	command = ItemFormatter::add_args(command);
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
pub fn execute(args: &ArgMatches) {
	let todo_filename = FileType::TodoTxt.filename(args);
	let list = List::from_url(todo_filename.clone())
		.expect("Could not read todo list");
	let search_terms = SearchTerms::from_argmatches(args);
	let mut formatter = ItemFormatter::from_argmatches(args);
	formatter.line_number_digits = list.lines.len().to_string().len();
	let confirmation = ConfirmationStatus::from_argmatches(args);
	let include_date = !*args.get_one::<bool>("no-date").unwrap();

	let (count, new_list) = mark_items_done_in_list(
		list,
		search_terms,
		&mut formatter,
		confirmation,
		include_date,
	);

	if count > 0 {
		new_list.to_url(todo_filename);
		println!("Marked {} tasks complete!", count);
	} else {
		println!("No actions taken.");
	}

	maybe_housekeeping_warnings(&new_list);
}

/// Return a new list with certain tasks in the given list marked as complete, based on the
/// search terms. Also returns a count of items modified.
pub fn mark_items_done_in_list(
	input: List,
	search_terms: SearchTerms,
	formatter: &mut ItemFormatter,
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
					&& check_if_complete(&item, formatter, status)
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
	formatter: &mut ItemFormatter,
	status: ConfirmationStatus,
) -> bool {
	formatter.write_item(item);
	status.check("Mark finished?", "Marking finished", "Skipping")
}

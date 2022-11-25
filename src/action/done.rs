use crate::action::*;
use crate::item::Item;
use crate::list::{LineKind, List};
use clap::{Arg, ArgMatches, Command};
use std::io;

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
	let mut formatter = ItemFormatter::from_argmatches(args);
	formatter.line_number_digits = list.lines.len().to_string().len();

	let search_terms = SearchTerms::from_argmatches(args);
	let mut new_list = List::new();

	let confirmation = ConfirmationStatus::from_argmatches(args);
	let include_date = !*args.get_one::<bool>("no-date").unwrap();

	let mut count = 0;
	for line in list.lines {
		match line.kind {
			LineKind::Item => {
				let item = line.item.clone().unwrap();
				if search_terms.item_matches(&item)
					&& (!item.completion())
					&& check_if_complete(
						&item,
						&formatter,
						&mut io::stdout(),
						confirmation,
					) {
					count += 1;
					new_list.lines.push(line.but_done(include_date));
				} else {
					new_list.lines.push(line);
				}
			}
			_ => new_list.lines.push(line),
		}
	}

	if count > 0 {
		new_list.to_url(todo_filename);
		println!("Marked {} tasks complete!", count);
	} else {
		println!("No actions taken.");
	}

	maybe_housekeeping_warnings(&new_list);
}

/// Asks whether to mark an item as complete, and prints out the response before returning a bool.
pub fn check_if_complete(
	item: &Item,
	formatter: &ItemFormatter,
	out: &mut std::io::Stdout,
	status: ConfirmationStatus,
) -> bool {
	formatter.write_item_to(item, out);
	status.check("Mark finished?", "Marking finished", "Skipping")
}

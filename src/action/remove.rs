//! Remove a task or tasks

use crate::action::*;
use crate::item::Item;
use crate::list::{Line, LineKind, List};
use clap::{ArgMatches, Command};

/// Options for the `remove` subcommand.
pub fn get_action() -> Action {
	let name = String::from("remove");
	let mut command = Command::new("remove")
		.aliases(["rm"])
		.about("Remove a task or tasks");

	command = FileType::TodoTxt.add_args(command);
	command = Outputter::add_args(command);
	command = SearchTerms::add_args(command);
	command = ConfirmationStatus::add_args(command);

	Action { name, command }
}

/// Execute the `remove` subcommand.
pub fn execute(args: &ArgMatches) {
	let todo_filename = FileType::TodoTxt.filename(args);
	let list = List::from_url(todo_filename.clone())
		.expect("Could not read todo list");

	let mut outputter = Outputter::from_argmatches(args);
	outputter.line_number_digits = list.lines.len().to_string().len();

	let search_terms = SearchTerms::from_argmatches(args);
	let mut new_list = List::new();

	let confirmation = ConfirmationStatus::from_argmatches(args);

	let mut count = 0;
	for line in list.lines {
		match line.kind {
			LineKind::Item => {
				let item = line.item.clone().unwrap();
				if search_terms.item_matches(&item)
					&& check_if_delete(&item, &mut outputter, confirmation)
				{
					count += 1;
					new_list.lines.push(Line::new_blank());
				} else {
					new_list.lines.push(line);
				}
			}
			_ => new_list.lines.push(line),
		}
	}

	if count > 0 {
		new_list.to_url(todo_filename);
		outputter.write_status(format!("Removed {} tasks!", count));
	} else {
		outputter.write_status(String::from("No actions taken."));
	}
}

/// Asks whether to delete an item, and prints out the response before returning a bool.
pub fn check_if_delete(
	item: &Item,
	outputter: &mut Outputter,
	status: ConfirmationStatus,
) -> bool {
	outputter.write_item(item);
	status.check(outputter, "Remove?", "Removing", "Keeping")
}

#[cfg(test)]
mod tests {
	use super::*;
	use tempfile::tempdir;

	#[test]
	fn test_get_action() {
		assert_eq!(String::from("remove"), get_action().name);
	}

	#[test]
	fn test_check_if_delete() {
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
		let r = check_if_delete(&i, &mut o, ConfirmationStatus::Yes);
		assert_eq!(true, r);
		let got_output = fs::read_to_string(buffer_filename.clone()).unwrap();
		assert_eq!(String::from("  (?) XYZ\nRemoving\n\n"), got_output);

		let mut o = Outputter::new(9999);
		o.colour = false;
		o.io = Box::new(fs::File::create(buffer_filename.clone()).unwrap());
		let r = check_if_delete(&i, &mut o, ConfirmationStatus::No);
		assert_eq!(false, r);
		let got_output = fs::read_to_string(buffer_filename.clone()).unwrap();
		assert_eq!(String::from("  (?) XYZ\nKeeping\n\n"), got_output);
	}
}

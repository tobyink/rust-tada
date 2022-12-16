//! Show the full todo list

use crate::action::*;
use crate::item::{TshirtSize, Urgency};
use crate::util::*;
use clap::{ArgMatches, Command};

/// Options for the `show` subcommand.
pub fn get_action() -> Action {
	let name = String::from("show");
	let mut command = Command::new("show").about("Show the full todo list");

	command = FileType::TodoTxt.add_args(command);
	command = Outputter::add_args(command);
	command = SortOrder::add_args(command, default_sort_order());
	command = Grouping::add_args(command);

	Action { name, command }
}

/// The default sort order for output.
pub fn default_sort_order() -> SortOrder {
	SortOrder::Smart
}

/// Execute the `show` subcommand.
pub fn execute(args: &ArgMatches) {
	let list = FileType::TodoTxt.load(args);
	let sort_order = SortOrder::from_argmatches(args, default_sort_order());
	let grouping = Grouping::from_argmatches(args);
	let mut outputter = Outputter::from_argmatches(args);
	outputter.line_number_digits = list.lines.len().to_string().len();

	show_list(&list, &grouping, &sort_order, &mut outputter);
	maybe_housekeeping_warnings(&mut outputter, &list);
}

/// Guts for the show command.
///
/// Outputs an entire todo list with a given grouping and sort order.
pub fn show_list(
	list: &List,
	grouping: &Grouping,
	sort_order: &SortOrder,
	outputter: &mut Outputter,
) {
	match grouping {
		Grouping::Urgency => {
			let split = group_items_by_urgency(list.items());
			for u in Urgency::all() {
				if let Some(items) = split.get(&u) {
					outputter.write_heading(String::from(u.to_string()));
					for i in sort_order.sort_items(items.to_vec()).iter() {
						outputter.write_item(i);
					}
					outputter.write_separator();
				}
			}
		}
		Grouping::Importance => {
			let split = group_items_by_importance(list.items());
			for u in Importance::all() {
				if let Some(items) = split.get(&u) {
					outputter.write_heading(String::from(u.to_string()));
					for i in sort_order.sort_items(items.to_vec()).iter() {
						outputter.write_item(i);
					}
					outputter.write_separator();
				}
			}
		}
		Grouping::TshirtSize => {
			let split = group_items_by_size(list.items());
			for u in TshirtSize::all() {
				if let Some(items) = split.get(&u) {
					outputter.write_heading(String::from(u.to_string()));
					for i in sort_order.sort_items(items.to_vec()).iter() {
						outputter.write_item(i);
					}
					outputter.write_separator();
				}
			}
		}
		Grouping::None => {
			for i in sort_order.sort_items(list.items()).iter() {
				outputter.write_item(i);
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_get_action() {
		assert_eq!(String::from("show"), get_action().name);
	}

	#[test]
	fn test_default_sort_order() {
		assert_eq!(SortOrder::Smart, default_sort_order());
	}
}

// TODO TEST: show_list()

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
	use crate::Line;
	use tempfile::tempdir;

	#[test]
	fn test_get_action() {
		assert_eq!(String::from("show"), get_action().name);
	}

	#[test]
	fn test_default_sort_order() {
		assert_eq!(SortOrder::Smart, default_sort_order());
	}

	#[test]
	fn test_show_list() {
		let dir = tempdir().unwrap();
		let buffer_filename = dir
			.path()
			.join("buffer.txt")
			.display()
			.to_string();

		let source_list = List {
			lines: Vec::from([
				Line::from_string(String::from("(A) 2000-01-01 Foo"), 1),
				Line::from_string(String::from("(B) Bar"), 2),
				Line::from_string(String::from("(D) Baz"), 3),
				Line::from_string(String::from("Bat"), 4),
			]),
			path: None,
		};

		let mut o = Outputter::new(9999);
		o.colour = false;
		o.io = Box::new(fs::File::create(buffer_filename.clone()).unwrap());
		show_list(&source_list, &Grouping::None, &SortOrder::Original, &mut o);
		let got_output = fs::read_to_string(buffer_filename.clone()).unwrap();
		assert_eq!(
			String::from(
				"  \
			(A) Foo\n  \
			(B) Bar\n  \
			(D) Baz\n  \
			(?) Bat\n"
			),
			got_output
		);

		let mut o = Outputter::new(9999);
		o.colour = false;
		o.io = Box::new(fs::File::create(buffer_filename.clone()).unwrap());
		show_list(
			&source_list,
			&Grouping::None,
			&SortOrder::Alphabetical,
			&mut o,
		);
		let got_output = fs::read_to_string(buffer_filename.clone()).unwrap();
		assert_eq!(
			String::from(
				"  \
			(B) Bar\n  \
			(?) Bat\n  \
			(D) Baz\n  \
			(A) Foo\n"
			),
			got_output
		);

		let mut o = Outputter::new(9999);
		o.colour = false;
		o.io = Box::new(fs::File::create(buffer_filename.clone()).unwrap());
		show_list(
			&source_list,
			&Grouping::Importance,
			&SortOrder::Alphabetical,
			&mut o,
		);
		let got_output = fs::read_to_string(buffer_filename.clone()).unwrap();
		assert_eq!(
			String::from(
				"\
			# Critical\n  \
			(A) Foo\n\n\
			# Important\n  \
			(B) Bar\n\n\
			# Normal\n  \
			(?) Bat\n  \
			(D) Baz\n\n"
			),
			got_output
		);

		let mut o = Outputter::new(9999);
		o.colour = false;
		o.with_creation_date = true;
		o.with_completion_date = true;
		o.with_line_numbers = true;
		o.io = Box::new(fs::File::create(buffer_filename.clone()).unwrap());
		show_list(
			&source_list,
			&Grouping::Importance,
			&SortOrder::Original,
			&mut o,
		);
		let got_output = fs::read_to_string(buffer_filename.clone()).unwrap();
		assert_eq!(
			String::from(
				"\
			# Critical\n  \
			(A)            2000-01-01 #01 Foo\n\n\
			# Important\n  \
			(B)            ????-??-?? #02 Bar\n\n\
			# Normal\n  \
			(D)            ????-??-?? #03 Baz\n  \
			(?)            ????-??-?? #04 Bat\n\n"
			),
			got_output
		);

		let mut o = Outputter::new(9999);
		o.colour = false;
		o.with_creation_date = true;
		o.with_completion_date = true;
		o.with_line_numbers = true;
		o.line_number_digits = 4;
		o.io = Box::new(fs::File::create(buffer_filename.clone()).unwrap());
		show_list(
			&source_list,
			&Grouping::Importance,
			&SortOrder::Original,
			&mut o,
		);
		let got_output = fs::read_to_string(buffer_filename.clone()).unwrap();
		assert_eq!(
			String::from(
				"\
			# Critical\n  \
			(A)            2000-01-01 #0001 Foo\n\n\
			# Important\n  \
			(B)            ????-??-?? #0002 Bar\n\n\
			# Normal\n  \
			(D)            ????-??-?? #0003 Baz\n  \
			(?)            ????-??-?? #0004 Bat\n\n"
			),
			got_output
		);
	}
}

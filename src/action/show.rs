//! Show the full todo list

use crate::action::*;
use crate::item::{TshirtSize, Urgency};
use crate::util::*;
use clap::{Arg, ArgMatches, Command};

/// Options for the `show` subcommand.
pub fn get_action() -> Action {
	let name = String::from("show");
	let mut command = Command::new("show").about("Show the full todo list");

	command = FileType::TodoTxt.add_args(command);
	command = Outputter::add_args(command);
	command = command
		.arg(
			Arg::new("sort")
				.num_args(1)
				.short('s')
				.long("sort")
				.value_name("BY")
				.help("Sort by 'smart' (default), 'urgency', 'importance', 'size', 'alpha', 'due', or 'orig'"),
		)
		.arg(
			Arg::new("importance")
				.num_args(0)
				.short('i')
				.long("importance")
				.aliases(["import", "imp", "important"])
				.help("Group by importance"),
		)
		.arg(
			Arg::new("urgency")
				.num_args(0)
				.short('u')
				.long("urgency")
				.aliases(["urgent", "urg"])
				.help("Group by urgency"),
		)
		.arg(
			Arg::new("size")
				.num_args(0)
				.short('z')
				.long("size")
				.aliases(["tshirt-size", "tshirt", "quick"])
				.help("Group by tshirt size"),
		);

	Action { name, command }
}

/// Execute the `show` subcommand.
pub fn execute(args: &ArgMatches) {
	let default_sort_by_type = String::from("smart");
	let sort_by_type = args
		.get_one::<String>("sort")
		.unwrap_or(&default_sort_by_type);

	let list = FileType::TodoTxt.load(args);

	let mut outputter = Outputter::from_argmatches(args);
	outputter.line_number_digits = list.lines.len().to_string().len();

	if *args.get_one::<bool>("urgency").unwrap() {
		let split = group_items_by_urgency(list.items());
		for u in Urgency::all() {
			if let Some(items) = split.get(&u) {
				outputter.write_heading(String::from(u.to_string()));
				for i in
					sort_items_by(sort_by_type.as_str(), items.to_vec()).iter()
				{
					outputter.write_item(i);
				}
				outputter.write_separator();
			}
		}
	} else if *args.get_one::<bool>("importance").unwrap() {
		let split = group_items_by_importance(list.items());
		for u in Importance::all() {
			if let Some(items) = split.get(&u) {
				outputter.write_heading(String::from(u.to_string()));
				for i in
					sort_items_by(sort_by_type.as_str(), items.to_vec()).iter()
				{
					outputter.write_item(i);
				}
				outputter.write_separator();
			}
		}
	} else if *args.get_one::<bool>("size").unwrap() {
		let split = group_items_by_size(list.items());
		for u in TshirtSize::all() {
			if let Some(items) = split.get(&u) {
				outputter.write_heading(String::from(u.to_string()));
				for i in
					sort_items_by(sort_by_type.as_str(), items.to_vec()).iter()
				{
					outputter.write_item(i);
				}
				outputter.write_separator();
			}
		}
	} else {
		for i in sort_items_by(sort_by_type.as_str(), list.items()).iter() {
			outputter.write_item(i);
		}
	}

	maybe_housekeeping_warnings(&mut outputter, &list);
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_get_action() {
		assert_eq!(String::from("show"), get_action().name);
	}
}

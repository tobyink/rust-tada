use crate::action::*;
use crate::item::{TshirtSize, Urgency, URGENCIES};
use crate::util::*;
use clap::{Arg, ArgMatches, Command};
use std::io;
use std::io::Write;

/// Options for the `show` subcommand.
pub fn get_action() -> Action {
	let name = String::from("show");
	let mut command = Command::new("show").about("Show the full todo list");

	command = FileType::TodoTxt.add_args(command);
	command = ItemFormatter::add_args(command);
	command = command
		.arg(
			Arg::new("sort")
				.num_args(1)
				.short('s')
				.long("sort")
				.value_name("BY")
				.help("sort by 'smart' (default), 'urgency', 'importance', 'size', 'alpha', 'due', or 'orig'"),
		)
		.arg(
			Arg::new("importance")
				.num_args(0)
				.short('i')
				.long("importance")
				.aliases(["import", "imp", "important"])
				.help("group by importance"),
		)
		.arg(
			Arg::new("urgency")
				.num_args(0)
				.short('u')
				.long("urgency")
				.aliases(["urgent", "urg"])
				.help("group by urgency"),
		)
		.arg(
			Arg::new("size")
				.num_args(0)
				.short('z')
				.long("size")
				.aliases(["tshirt-size", "tshirt", "quick"])
				.help("group by tshirt size"),
		);

	Action { name, command }
}

/// Execute the `show` subcommand.
pub fn execute(args: &ArgMatches) {
	let default_sort_by_type = String::from("smart");
	let sort_by_type = args
		.get_one::<String>("sort")
		.unwrap_or(&default_sort_by_type);

	let mut out = io::stdout();
	let list = FileType::TodoTxt.load(args);

	let mut formatter = ItemFormatter::from_argmatches(args);
	formatter.line_number_digits = list.lines.len().to_string().len();

	if *args.get_one::<bool>("urgency").unwrap() {
		let split = group_items_by_urgency(list.items());
		for u in URGENCIES.iter() {
			if let Some(items) = split.get(u) {
				let label = match u {
					Urgency::Overdue => "Overdue",
					Urgency::Today => "Today",
					Urgency::Soon => "Soon",
					Urgency::ThisWeek => "This week",
					Urgency::NextWeek => "Next week",
					Urgency::NextMonth => "Next month",
					Urgency::Later => "Later",
				};
				writeln!(&out, "# {}", label).expect("panik");
				for i in
					sort_items_by(sort_by_type.as_str(), items.to_vec()).iter()
				{
					formatter.write_item_to(i, &mut out);
				}
				writeln!(&out).expect("panik");
			}
		}
	} else if *args.get_one::<bool>("importance").unwrap() {
		let split = group_items_by_importance(list.items());
		for u in ['A', 'B', 'C', 'D', 'E'] {
			if let Some(items) = split.get(&u) {
				let label = match u {
					'A' => "Critical",
					'B' => "Important",
					'C' => "Semi-important",
					'D' => "Normal",
					_ => "Unimportant",
				};
				writeln!(&out, "# {}", label).expect("panik");
				for i in
					sort_items_by(sort_by_type.as_str(), items.to_vec()).iter()
				{
					formatter.write_item_to(i, &mut out);
				}
				writeln!(&out).expect("panik");
			}
		}
	} else if *args.get_one::<bool>("size").unwrap() {
		let split = group_items_by_size(list.items());
		for u in [TshirtSize::Small, TshirtSize::Medium, TshirtSize::Large] {
			if let Some(items) = split.get(&u) {
				let label = match u {
					TshirtSize::Small => "Small",
					TshirtSize::Medium => "Medium",
					TshirtSize::Large => "Large",
				};
				writeln!(&out, "# {}", label).expect("panik");
				for i in
					sort_items_by(sort_by_type.as_str(), items.to_vec()).iter()
				{
					formatter.write_item_to(i, &mut out);
				}
				writeln!(&out).expect("panik");
			}
		}
	} else {
		for i in sort_items_by(sort_by_type.as_str(), list.items()).iter() {
			formatter.write_item_to(i, &mut out);
		}
	}

	maybe_housekeeping_warnings(&list);
}

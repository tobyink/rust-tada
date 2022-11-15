use crate::*;
use clap::{Arg, ArgMatches, Command};
use std::collections::HashMap;
use std::io;
use std::io::Write;

/// Options for the `show` subcommand.
pub fn get_action() -> Action {
	let name = String::from("show");
	let mut command = Command::new("show").about("Show the full todo list");

	command = Action::_add_todotxt_file_options(command);
	command = Action::_add_output_options(command);
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
				.help("group by importance"),
		)
		.arg(
			Arg::new("urgency")
				.num_args(0)
				.short('u')
				.long("urgency")
				.help("group by urgency"),
		)
		.arg(
			Arg::new("size")
				.num_args(0)
				.short('z')
				.long("size")
				.aliases(["tshirt-size", "tshirt" ])
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
	let cfg = Action::build_output_config(args);
	let list =
		List::from_url(Action::determine_filename(FileType::TodoTxt, args));

	if *args.get_one::<bool>("urgency").unwrap() {
		let split = group_by_urgency(list.items());
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
					Action::sort_items_by(sort_by_type.as_str(), items.to_vec())
						.iter()
				{
					i.write_to(&mut out, &cfg);
				}
				writeln!(&out).expect("panik");
			}
		}
	} else if *args.get_one::<bool>("importance").unwrap() {
		let split = group_by_importance(list.items());
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
					Action::sort_items_by(sort_by_type.as_str(), items.to_vec())
						.iter()
				{
					i.write_to(&mut out, &cfg);
				}
				writeln!(&out).expect("panik");
			}
		}
	} else if *args.get_one::<bool>("size").unwrap() {
		let split = group_by_size(list.items());
		for u in [TshirtSize::Small, TshirtSize::Medium, TshirtSize::Large] {
			if let Some(items) = split.get(&u) {
				let label = match u {
					TshirtSize::Small => "Small",
					TshirtSize::Medium => "Medium",
					TshirtSize::Large => "Large",
				};
				writeln!(&out, "# {}", label).expect("panik");
				for i in
					Action::sort_items_by(sort_by_type.as_str(), items.to_vec())
						.iter()
				{
					i.write_to(&mut out, &cfg);
				}
				writeln!(&out).expect("panik");
			}
		}
	} else {
		for i in
			Action::sort_items_by(sort_by_type.as_str(), list.items()).iter()
		{
			i.write_to(&mut out, &cfg);
		}
	}
}

pub fn group_by_urgency(items: Vec<&Item>) -> HashMap<Urgency, Vec<&Item>> {
	let mut out: HashMap<Urgency, Vec<&Item>> = HashMap::new();
	for i in items {
		let list = out
			.entry(i.urgency().unwrap_or(Urgency::Soon))
			.or_insert_with(Vec::new);
		list.push(i);
	}
	out
}

pub fn group_by_size(items: Vec<&Item>) -> HashMap<TshirtSize, Vec<&Item>> {
	let mut out: HashMap<TshirtSize, Vec<&Item>> = HashMap::new();
	for i in items {
		let list = out
			.entry(i.tshirt_size().unwrap_or(TshirtSize::Medium))
			.or_insert_with(Vec::new);
		list.push(i);
	}
	out
}

pub fn group_by_importance(items: Vec<&Item>) -> HashMap<char, Vec<&Item>> {
	let mut out: HashMap<char, Vec<&Item>> = HashMap::new();
	for i in items {
		let list = out
			.entry(i.importance().unwrap_or('D'))
			.or_insert_with(Vec::new);
		list.push(i);
	}
	out
}

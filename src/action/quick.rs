use crate::action::*;
use crate::util::*;
use clap::{Arg, ArgMatches, Command};
use std::io;

/// Options for the `quick` subcommand.
pub fn get_action() -> Action {
	let name = String::from("quick");
	let mut command = Command::new("quick")
		.aliases(["q"])
		.about("Show the smallest tasks")
		.after_help(
			"Ignores tasks which are marked as already complete or \
			have a start date in the future.",
		);

	command = FileType::TodoTxt.add_args(command);
	command = ItemFormatter::add_args(command);
	command = command
		.arg(
			Arg::new("number")
				.num_args(1)
				.short('n')
				.long("number")
				.value_parser(clap::value_parser!(usize))
				.value_name("N")
				.help("maximum number to show (default: 3)"),
		)
		.arg(
			Arg::new("sort")
				.num_args(1)
				.short('s')
				.long("sort")
				.value_name("BY")
				.help("sort by 'smart', 'urgency', 'importance' (default), 'size', 'alpha', or 'due'"),
		);

	Action { name, command }
}

/// Execute the `important` subcommand.
pub fn execute(args: &ArgMatches) {
	let default_sort_by_type = String::from("importance");
	let sort_by_type = args
		.get_one::<String>("sort")
		.unwrap_or(&default_sort_by_type);
	let max = args.get_one::<usize>("number").unwrap_or(&3);

	let mut out = io::stdout();
	let list = FileType::TodoTxt.load(args);

	let mut formatter = ItemFormatter::from_argmatches(args);
	formatter.line_number_digits = list.lines.len().to_string().len();

	let quick = sort_items_by("size", list.items())
		.into_iter()
		.filter(|i| i.is_startable() && !i.completion())
		.take(*max)
		.collect();

	for i in sort_items_by(sort_by_type.as_str(), quick).iter() {
		formatter.write_item_to(i, &mut out);
	}
}

use crate::*;
use clap::{Arg, ArgMatches, Command};
use std::io;

/// Options for the `urgent` subcommand.
pub fn get_action() -> Action {
	let name = String::from("urgent");
	let mut command = Command::new("urgent")
		.about("Show the most urgent tasks")
		.after_help("Ignores tasks which are marked as already complete.");

	command = Action::_add_todotxt_file_options(command);
	command = Action::_add_output_options(command);
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
				.help("sort by 'smart', 'urgency', 'importance', 'size', 'alpha', 'due' (default), or 'orig'"),
		);

	Action { name, command }
}

/// Execute the `urgent` subcommand.
pub fn execute(args: &ArgMatches) {
	let default_sort_by_type = String::from("due");
	let sort_by_type = args
		.get_one::<String>("sort")
		.unwrap_or(&default_sort_by_type);
	let max = args.get_one::<usize>("number").unwrap_or(&3);

	let mut out = io::stdout();
	let cfg = Action::build_output_config(args);
	let list =
		List::from_url(Action::determine_filename(FileType::TodoTxt, args));
	let urgent = Action::sort_items_by("due", list.items())
		.into_iter()
		.filter(|i| !i.completion())
		.take(*max)
		.collect();

	for i in Action::sort_items_by(sort_by_type.as_str(), urgent).iter() {
		i.write_to(&mut out, &cfg);
	}
}

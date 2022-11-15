use crate::*;
use clap::{Arg, ArgMatches, Command};
use std::io;

/// Options for the `important` subcommand.
pub fn get_action() -> Action {
	let name = String::from("important");
	let mut command = Command::new("important")
		.about("Show the most important tasks")
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
				.help("sort by 'smart', 'urgency', 'importance' (default), 'size', 'alpha', 'due', or 'orig'"),
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
	let cfg = Action::build_output_config(args);
	let list =
		List::from_url(Action::determine_filename(FileType::TodoTxt, args));
	let important = Action::sort_items_by("importance", list.items())
		.into_iter()
		.filter(|i| !i.completion())
		.take(*max)
		.collect();

	for i in Action::sort_items_by(sort_by_type.as_str(), important).iter() {
		i.write_to(&mut out, &cfg);
	}
}

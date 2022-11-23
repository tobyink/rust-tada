use crate::action::{Action, FileType};
use crate::list::List;
use crate::util::*;
use clap::{Arg, ArgMatches, Command};
use std::io;

/// Options for the `important` subcommand.
pub fn get_action() -> Action {
	let name = String::from("important");
	let mut command = Command::new("important")
		.aliases(["i"])
		.about("Show the most important tasks")
		.after_help(
			"Ignores tasks which are marked as already complete or \
			have a start date in the future.",
		);

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
	let list =
		List::from_url(Action::determine_filename(FileType::TodoTxt, args))
			.expect("Could not read todo list");
	let mut cfg = Action::build_output_config(args);
	cfg.line_number_digits = list.lines.len().to_string().len();

	let important = sort_items_by("importance", list.items())
		.into_iter()
		.filter(|i| i.is_startable() && !i.completion())
		.take(*max)
		.collect();

	for i in sort_items_by(sort_by_type.as_str(), important).iter() {
		i.write_to(&mut out, &cfg);
	}
}

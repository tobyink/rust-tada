use crate::util::*;
use crate::*;
use clap::{Arg, ArgAction, ArgMatches, Command};
use std::io;

/// Options for the `find` subcommand.
pub fn get_action() -> Action {
	let name = String::from("find");
	let mut command = Command::new("find").about("Search for a task")
		.after_help(
			"Multiple search terms may be provided, which will be combined with an 'AND' operator.\n\n\
			Searches are case-insensitive."
		);

	command = Action::_add_todotxt_file_options(command);
	command = Action::_add_output_options(command);
	command = command
		.arg(
			Arg::new("search-term")
				.action(ArgAction::Append)
				.required(true)
				.help("a tag, context, line number, or string"),
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

/// Execute the `find` subcommand.
pub fn execute(args: &ArgMatches) {
	let default_sort_by_type = String::from("smart");
	let sort_by_type = args
		.get_one::<String>("sort")
		.unwrap_or(&default_sort_by_type);

	let mut out = io::stdout();
	let list =
		List::from_url(Action::determine_filename(FileType::TodoTxt, args))
			.expect("Could not read todo list");
	let mut results = list.items();
	let mut cfg = Action::build_output_config(args);
	cfg.line_number_digits = list.lines.len().to_string().len();

	for term in args.get_many::<String>("search-term").unwrap() {
		results = match term.chars().next() {
			Some('@') => find_by_context(term, results),
			Some('+') => find_by_tag(term, results),
			Some('#') => find_by_line_number(term, results),
			_ => find_by_string(term, results),
		};
	}

	for i in sort_items_by(sort_by_type.as_str(), results).iter() {
		i.write_to(&mut out, &cfg);
	}
}

/// Execute the `find` subcommand via shortcut.
pub fn execute_shortcut(term: &str) {
	let cmd = get_action().command;
	let matches = cmd.get_matches_from(vec!["find", term]);
	execute(&matches);
}

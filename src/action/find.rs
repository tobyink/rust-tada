use crate::action::*;
use crate::util::*;
use clap::{Arg, ArgMatches, Command};

/// Options for the `find` subcommand.
pub fn get_action() -> Action {
	let name = String::from("find");
	let mut command = Command::new("find").about("Search for a task")
		.after_help(
			"Multiple search terms may be provided, which will be combined with an 'AND' operator.\n\n\
			Searches are case-insensitive."
		);

	command = FileType::TodoTxt.add_args(command);
	command = Outputter::add_args(command);
	command = SearchTerms::add_args(command);
	command = command
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

	let list = FileType::TodoTxt.load(args);
	let mut results = list.items();
	let mut outputter = Outputter::from_argmatches(args);
	outputter.line_number_digits = list.lines.len().to_string().len();

	let search_terms = SearchTerms::from_argmatches(args);
	for term in &search_terms.terms {
		results = match term.chars().next() {
			Some('@') => find_items_by_context(term, results),
			Some('+') => find_items_by_tag(term, results),
			Some('#') => find_items_by_line_number(term, results),
			_ => find_items_by_string(term, results),
		};
	}

	for i in sort_items_by(sort_by_type.as_str(), results).iter() {
		outputter.write_item(i);
	}
}

/// Execute the `find` subcommand via shortcut.
pub fn execute_shortcut(term: &str) {
	let cmd = get_action().command;
	let matches = cmd.get_matches_from(vec!["find", term]);
	execute(&matches);
}

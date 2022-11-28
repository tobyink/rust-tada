use crate::action::*;
use crate::util::*;
use clap::{ArgMatches, Command};

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
	command = SortOrder::add_args(command, default_sort_order());

	Action { name, command }
}

pub fn default_sort_order() -> &'static str {
	"smart"
}

/// Execute the `find` subcommand.
#[cfg(not(tarpaulin_include))]
pub fn execute(args: &ArgMatches) {
	let list = FileType::TodoTxt.load(args);

	let mut outputter = Outputter::from_argmatches(args);
	outputter.line_number_digits = list.lines.len().to_string().len();

	let search_terms = SearchTerms::from_argmatches(args);
	let results = find_results(&search_terms, &list);
	let sort_order = SortOrder::from_argmatches(args, default_sort_order());

	for i in sort_order.sort_items(results).iter() {
		outputter.write_item(i);
	}
}

/// Execute the `find` subcommand via shortcut.
#[cfg(not(tarpaulin_include))]
pub fn execute_shortcut(term: &str) {
	let cmd = get_action().command;
	let matches = cmd.get_matches_from(vec!["find", term]);
	execute(&matches);
}

/// Given search terms and a list, returns items from the list matching the search terms.
///
/// If there is more than one search term, then each item returned will match all terms.
/// That is, the search terms are combined with an AND operator, not an OR operator.
pub fn find_results<'a, 'b: 'a>(
	search_terms: &'a SearchTerms,
	list: &'b List,
) -> Vec<&'a Item> {
	let mut results = list.items();
	for term in &search_terms.terms {
		results = match term.chars().next() {
			Some('@') => find_items_by_context(term, results),
			Some('+') => find_items_by_tag(term, results),
			Some('#') => find_items_by_line_number(term, results),
			_ => find_items_by_string(term, results),
		};
	}
	results
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_get_action() {
		assert_eq!(String::from("find"), get_action().name);
	}

	#[test]
	fn test_default_sort_order() {
		assert_eq!("smart", default_sort_order());
	}

	#[test]
	fn test_find_results() {
		let list = List::from_string(
			"Foo\n\
			Foo bar\n\
			Bar\n\
			#Foo\n\
			+foo\n\
			(A) @foo\n\
			"
			.to_string(),
		)
		.unwrap();

		let t = SearchTerms::from_string("Foo");
		assert_eq!(
			"Foo\n\
			Foo bar\n\
			+foo\n\
			(A) @foo\n\
			",
			List::from_items(find_results(&t, &list)).serialize(),
		);

		let t = SearchTerms::from_string("@Foo");
		assert_eq!(
			"(A) @foo\n\
			",
			List::from_items(find_results(&t, &list)).serialize(),
		);

		let t = SearchTerms::from_string("+Foo");
		assert_eq!(
			"+foo\n\
			",
			List::from_items(find_results(&t, &list)).serialize(),
		);

		let t = SearchTerms::from_string("BAR");
		assert_eq!(
			"Foo bar\n\
			Bar\n\
			",
			List::from_items(find_results(&t, &list)).serialize(),
		);

		let t = SearchTerms::from_vec(Vec::from([
			String::from("BAR"),
			String::from("fOO"),
		]));
		assert_eq!(
			"Foo bar\n\
			",
			List::from_items(find_results(&t, &list)).serialize(),
		);

		let t = SearchTerms::from_string("baz");
		assert_eq!("", List::from_items(find_results(&t, &list)).serialize());
	}
}

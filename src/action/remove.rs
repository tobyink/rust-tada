use crate::action::{Action, FileType};
use crate::item::{Item, ItemFormatConfig};
use crate::list::{Line, LineKind, List};
use clap::{Arg, ArgAction, ArgMatches, Command};
use promptly::prompt_default;
use std::io;

/// Options for the `remove` subcommand.
pub fn get_action() -> Action {
	let name = String::from("remove");
	let mut command = Command::new("remove").about("Remove a task or tasks");

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
			Arg::new("yes")
				.num_args(0)
				.short('y')
				.long("yes")
				.help("assume 'yes' to prompts"),
		)
		.arg(
			Arg::new("no")
				.num_args(0)
				.short('n')
				.long("no")
				.help("assume 'no' to prompts"),
		);

	Action { name, command }
}

/// Execute the `remove` subcommand.
pub fn execute(args: &ArgMatches) {
	let todo_filename = Action::determine_filename(FileType::TodoTxt, args);
	let list = List::from_url(todo_filename.clone())
		.expect("Could not read todo list");
	let mut cfg = Action::build_output_config(args);
	cfg.line_number_digits = list.lines.len().to_string().len();

	let terms: Vec<&String> = args
		.get_many::<String>("search-term")
		.unwrap()
		.collect();
	let mut new_list = List::new();

	let opt = if *args.get_one::<bool>("no").unwrap() {
		'N'
	} else if *args.get_one::<bool>("yes").unwrap() {
		'Y'
	} else {
		'?'
	};

	let mut count = 0;
	for line in list.lines {
		match line.kind {
			LineKind::Item => {
				let item = line.item.clone().unwrap();
				if item_matches_terms(&item, &terms)
					&& check_if_delete(&item, opt, &mut io::stdout(), &cfg)
				{
					count += 1;
					new_list.lines.push(Line::new_blank());
				} else {
					new_list.lines.push(line);
				}
			}
			_ => new_list.lines.push(line),
		}
	}

	if count > 0 {
		new_list.to_url(todo_filename);
		println!("Removed {} tasks!", count);
	} else {
		println!("No actions taken.");
	}
}

pub fn item_matches_terms(item: &Item, terms: &Vec<&String>) -> bool {
	for term in terms {
		match term.chars().next() {
			Some('@') => {
				if item.has_context(term) {
					return true;
				}
			}
			Some('+') => {
				if item.has_tag(term) {
					return true;
				}
			}
			Some('#') => {
				let n: usize = term.get(1..).unwrap().parse().unwrap();
				if item.line_number() == n {
					return true;
				}
			}
			_ => {
				let lc_term = term.to_lowercase();
				if item
					.description()
					.to_lowercase()
					.contains(&lc_term)
				{
					return true;
				}
			}
		}
	}
	false
}

pub fn check_if_delete(
	item: &Item,
	opt: char,
	out: &mut std::io::Stdout,
	cfg: &ItemFormatConfig,
) -> bool {
	item.write_to(out, cfg);

	if opt == 'Y' {
		println!("Removing\n");
		return true;
	}

	if opt == 'N' {
		println!("Keeping\n");
		return false;
	}

	let response = prompt_default("Remove?", true).unwrap();
	if response {
		println!("Removing\n");
		return true;
	}

	println!("Keeping\n");
	false
}

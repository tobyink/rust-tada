use crate::action::{Action, FileType};
use crate::item::{Item, ItemFormatConfig};
use crate::list::{LineKind, List};
use clap::{Arg, ArgAction, ArgMatches, Command};
use promptly::prompt_default;
use std::io;

/// Options for the `done` subcommand.
pub fn get_action() -> Action {
	let name = String::from("done");
	let mut command =
		Command::new("done").about("Mark a task or tasks as done");

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
			Arg::new("no-date")
				.num_args(0)
				.long("no-date")
				.aliases(["nodate"])
				.help("Don't automatically add a completion date to the task"),
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

/// Execute the `done` subcommand.
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

	let include_date = !*args.get_one::<bool>("no-date").unwrap();

	let mut count = 0;
	for line in list.lines {
		match line.kind {
			LineKind::Item => {
				let item = line.item.clone().unwrap();
				if Action::item_matches_terms(&item, &terms)
					&& (!item.completion())
					&& check_if_complete(&item, opt, &mut io::stdout(), &cfg)
				{
					count += 1;
					new_list.lines.push(line.but_done(include_date));
				} else {
					new_list.lines.push(line);
				}
			}
			_ => new_list.lines.push(line),
		}
	}

	if count > 0 {
		new_list.to_url(todo_filename);
		println!("Marked {} tasks complete!", count);
	} else {
		println!("No actions taken.");
	}
}

/// Asks whether to mark an item as complete, and prints out the response before returning a bool.
pub fn check_if_complete(
	item: &Item,
	opt: char,
	out: &mut std::io::Stdout,
	cfg: &ItemFormatConfig,
) -> bool {
	item.write_to(out, cfg);

	if opt == 'Y' {
		println!("Marking finished\n");
		return true;
	}

	if opt == 'N' {
		println!("Skipping\n");
		return false;
	}

	let response = prompt_default("Mark finished?", true).unwrap();
	if response {
		println!("Marking finished\n");
		return true;
	}

	println!("Skipping\n");
	false
}

use crate::action::{Action, ConfirmationStatus, FileType};
use crate::item::{Item, ItemFormatConfig};
use crate::list::{LineKind, List};
use clap::{Arg, ArgMatches, Command};
use std::io;

/// Options for the `done` subcommand.
pub fn get_action() -> Action {
	let name = String::from("done");
	let mut command =
		Command::new("done").about("Mark a task or tasks as done");

	command = Action::_add_todotxt_file_options(command);
	command = Action::_add_output_options(command);
	command = Action::_add_search_terms_option(command);
	command = command
		.arg(
			Arg::new("no-date")
				.num_args(0)
				.long("no-date")
				.aliases(["nodate"])
				.help("Don't automatically add a completion date to the task"),
		);
	command = Action::_add_prompt_options(command);

	Action { name, command }
}

/// Execute the `done` subcommand.
pub fn execute(args: &ArgMatches) {
	let todo_filename = Action::determine_filename(FileType::TodoTxt, args);
	let list = List::from_url(todo_filename.clone())
		.expect("Could not read todo list");
	let mut cfg = Action::build_output_config(args);
	cfg.line_number_digits = list.lines.len().to_string().len();

	let terms = Action::determine_search_terms(args);
	let mut new_list = List::new();

	let confirmation = ConfirmationStatus::from_argmatches(args);
	let include_date = !*args.get_one::<bool>("no-date").unwrap();

	let mut count = 0;
	for line in list.lines {
		match line.kind {
			LineKind::Item => {
				let item = line.item.clone().unwrap();
				if Action::item_matches_terms(&item, &terms)
					&& (!item.completion())
					&& check_if_complete(
						&item,
						&mut io::stdout(),
						&cfg,
						confirmation,
					) {
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

	Action::maybe_warnings(&new_list);
}

/// Asks whether to mark an item as complete, and prints out the response before returning a bool.
pub fn check_if_complete(
	item: &Item,
	out: &mut std::io::Stdout,
	cfg: &ItemFormatConfig,
	status: ConfirmationStatus,
) -> bool {
	item.write_to(out, cfg);
	status.check("Mark finished?", "Marking finished", "Skipping")
}

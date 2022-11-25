use crate::action::{Action, ConfirmationStatus, FileType};
use crate::item::{Item, ItemFormatConfig};
use crate::list::{Line, LineKind, List};
use clap::{ArgMatches, Command};
use std::io;

/// Options for the `remove` subcommand.
pub fn get_action() -> Action {
	let name = String::from("remove");
	let mut command = Command::new("remove")
		.aliases(["rm"])
		.about("Remove a task or tasks");

	command = Action::_add_todotxt_file_options(command);
	command = Action::_add_output_options(command);
	command = Action::_add_search_terms_option(command);
	command = Action::_add_prompt_options(command);

	Action { name, command }
}

/// Execute the `remove` subcommand.
pub fn execute(args: &ArgMatches) {
	let todo_filename = Action::determine_filename(FileType::TodoTxt, args);
	let list = List::from_url(todo_filename.clone())
		.expect("Could not read todo list");
	let mut cfg = Action::build_output_config(args);
	cfg.line_number_digits = list.lines.len().to_string().len();

	let terms = Action::determine_search_terms(args);
	let mut new_list = List::new();

	let confirmation = ConfirmationStatus::from_argmatches(args);

	let mut count = 0;
	for line in list.lines {
		match line.kind {
			LineKind::Item => {
				let item = line.item.clone().unwrap();
				if Action::item_matches_terms(&item, &terms)
					&& check_if_delete(
						&item,
						&mut io::stdout(),
						&cfg,
						confirmation,
					) {
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

/// Asks whether to delete an item, and prints out the response before returning a bool.
pub fn check_if_delete(
	item: &Item,
	out: &mut std::io::Stdout,
	cfg: &ItemFormatConfig,
	status: ConfirmationStatus,
) -> bool {
	item.write_to(out, cfg);
	status.check("Remove?", "Removing", "Keeping")
}

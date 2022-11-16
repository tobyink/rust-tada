use crate::*;
use clap::{ArgMatches, Command};

/// Options for the `archive` subcommand.
pub fn get_action() -> Action {
	let name = String::from("archive");
	let mut command = Command::new("archive")
		.about("Move completed items from todo.txt to done.txt");

	command = Action::_add_todotxt_file_options(command);
	command = Action::_add_donetxt_file_options(command);

	Action { name, command }
}

/// Execute the `archive` subcommand.
pub fn execute(args: &ArgMatches) {
	let todo_filename = Action::determine_filename(FileType::TodoTxt, args);
	let done_filename = Action::determine_filename(FileType::DoneTxt, args);

	let mut new_todo: Vec<Line> = Vec::new();
	let mut append_done: Vec<Line> = Vec::new();

	let mut moved = 0;
	let todo = List::from_url(todo_filename.clone())
		.expect("Could not read todo list");
	for line in todo.lines {
		match line.kind {
			LineKind::Blank => new_todo.push(line),
			LineKind::Comment => new_todo.push(line),
			LineKind::Item => {
				let item = line.item.as_ref().expect("Expected a task!");
				if item.completion() {
					let new = Line {
						kind: LineKind::Item,
						text: line.text.clone(),
						item: Some(item.clone()),
					};
					moved += 1;
					append_done.push(new)
				} else {
					new_todo.push(line)
				}
			}
		}
	}

	if moved == 0 {
		println!("No complete tasks found in {}", todo_filename);
	} else {
		List::append_lines_to_url(
			done_filename.clone(),
			append_done.iter().collect(),
		);

		let mut list = List::new();
		list.lines = new_todo;
		list.to_url(todo_filename);

		println!("Moved {} tasks to {}", moved, done_filename);
	}
}

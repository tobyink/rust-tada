use crate::action::*;
use crate::list::{Line, LineKind, List};
use clap::{ArgMatches, Command};

/// Options for the `archive` subcommand.
pub fn get_action() -> Action {
	let name = String::from("archive");
	let mut command = Command::new("archive")
		.about("Move completed tasks from todo.txt to done.txt");

	command = FileType::TodoTxt.add_args(command);
	command = FileType::DoneTxt.add_args(command);

	Action { name, command }
}

/// Execute the `archive` subcommand.
#[cfg(not(tarpaulin_include))]
pub fn execute(args: &ArgMatches) {
	let todo_filename = FileType::TodoTxt.filename(args);
	let done_filename = FileType::DoneTxt.filename(args);
	let (num, result) = run_archive(&todo_filename, &done_filename);

	if num > 0 {
		println!("Moved {} tasks to {}", num, done_filename);
	} else {
		println!("No complete tasks found in {}", todo_filename);
	}

	maybe_housekeeping_warnings(&result);
}

/// Logic of archiving a todo.txt to a done.txt.
///
/// Will read the todo.txt and if there are any completed tasks in it, replace them
/// with blank lines (overwriting the original file), and append those completed tasks
/// to the done.txt.
///
/// If there are no completed tasks in the todo.txt, neither file should be written to.
///
/// Returns a tuple of the number of moved lines and the modified todo list.
pub fn run_archive(todo_filename: &str, done_filename: &str) -> (i32, List) {
	let todo = List::from_url(String::from(todo_filename))
		.expect("Could not read todo list");
	let mut new_todo: Vec<Line> = Vec::new();
	let mut append_done: Vec<Line> = Vec::new();

	let orig = todo.lines.clone();
	let moved = run_archive_vec(&orig, &mut new_todo, &mut append_done);

	if moved == 0 {
		return (moved, todo);
	}

	List::append_lines_to_url(
		String::from(done_filename),
		append_done.iter().collect(),
	);
	let mut list = List::new();
	list.lines = new_todo;
	list.to_url(String::from(todo_filename));
	(moved, list)
}

/// Logic of archiving a todo.txt to a done.txt, but with Vec<Line>.
///
/// Returns the number of lines archived.
pub fn run_archive_vec(
	src: &Vec<Line>,
	todo: &mut Vec<Line>,
	done: &mut Vec<Line>,
) -> i32 {
	let mut moved = 0;
	for line in src {
		match line.kind {
			LineKind::Blank => todo.push(line.clone()),
			LineKind::Comment => todo.push(line.clone()),
			LineKind::Item => {
				let item = line.item.as_ref().expect("Expected a task!");
				if item.completion() {
					let new = Line {
						kind: LineKind::Item,
						text: line.text.clone(),
						item: Some(item.clone()),
						num: 0,
					};
					moved += 1;
					done.push(new);
					todo.push(Line::new_blank())
				} else {
					todo.push(line.clone())
				}
			}
		}
	}
	moved
}

#[cfg(test)]
mod tests {
	use super::*;
	use tempfile::tempdir;

	#[test]
	fn test_get_action() {
		assert_eq!(String::from("archive"), get_action().name);
	}

	fn _eq_vecline(vl1: Vec<Line>, vl2: Vec<Line>) -> bool {
		let mut l1 = List::new();
		l1.lines = vl1;
		let mut l2 = List::new();
		l2.lines = vl2;
		l1.serialize() == l2.serialize()
	}

	#[test]
	pub fn test_run_archive_vec() {
		let source: Vec<Line> = Vec::from([
			Line::from_string(String::from("x Foo1"), 0),
			Line::from_string(String::from("x Foo2"), 0),
			Line::from_string(String::from(""), 0),
			Line::from_string(String::from("# Bar"), 0),
			Line::from_string(String::from(""), 0),
			Line::from_string(String::from("Z Baz"), 0),
		]);

		let expected_moved = 2;
		let expected_keep = Vec::from([
			Line::from_string(String::from(""), 0),
			Line::from_string(String::from(""), 0),
			Line::from_string(String::from(""), 0),
			Line::from_string(String::from("# Bar"), 0),
			Line::from_string(String::from(""), 0),
			Line::from_string(String::from("Z Baz"), 0),
		]);
		let expected_archive: Vec<Line> = Vec::from([
			Line::from_string(String::from("x Foo1"), 0),
			Line::from_string(String::from("x Foo2"), 0),
		]);

		let mut keep: Vec<Line> = Vec::new();
		let mut archive: Vec<Line> = Vec::new();
		let moved = run_archive_vec(&source, &mut keep, &mut archive);
		assert_eq!(expected_moved, moved);
		assert!(_eq_vecline(expected_keep, keep));
		assert!(_eq_vecline(expected_archive, archive));
	}

	#[test]
	pub fn test_run_archive() {
		let initial_todo: Vec<Line> = Vec::from([
			Line::from_string(String::from("x Foo1"), 0),
			Line::from_string(String::from("x Foo2"), 0),
			Line::from_string(String::from(""), 0),
			Line::from_string(String::from("# Bar"), 0),
			Line::from_string(String::from(""), 0),
			Line::from_string(String::from("Z Baz"), 0),
		]);
		let initial_done: Vec<Line> = Vec::from([
			Line::from_string(String::from("x Old1"), 0),
			Line::from_string(String::from("x Old2"), 0),
		]);

		let expected_moved = 2;
		let expected_todo = Vec::from([
			Line::from_string(String::from(""), 0),
			Line::from_string(String::from(""), 0),
			Line::from_string(String::from(""), 0),
			Line::from_string(String::from("# Bar"), 0),
			Line::from_string(String::from(""), 0),
			Line::from_string(String::from("Z Baz"), 0),
		]);
		let expected_done: Vec<Line> = Vec::from([
			Line::from_string(String::from("x Old1"), 0),
			Line::from_string(String::from("x Old2"), 0),
			Line::from_string(String::from("x Foo1"), 0),
			Line::from_string(String::from("x Foo2"), 0),
		]);

		let dir = tempdir().unwrap();

		let todo_filename = dir
			.path()
			.join("todo-X88.txt")
			.as_path()
			.display()
			.to_string();
		{
			let mut l = List::new();
			l.lines = initial_todo;
			l.to_filename(todo_filename.clone());
		}

		let done_filename = dir
			.path()
			.join("done-X88.txt")
			.as_path()
			.display()
			.to_string();
		{
			let mut l = List::new();
			l.lines = initial_done;
			l.to_filename(done_filename.clone());
		}

		let (moved, result) = run_archive(&todo_filename, &done_filename);
		assert_eq!(expected_moved, moved);
		assert!(_eq_vecline(result.lines, expected_todo.clone()));
		assert!(_eq_vecline(
			List::from_filename(todo_filename.clone())
				.unwrap()
				.lines,
			expected_todo
		));
		assert!(_eq_vecline(
			List::from_filename(done_filename.clone())
				.unwrap()
				.lines,
			expected_done
		));
	}

	#[test]
	pub fn test_run_archive_but_nothing_done() {
		let initial_todo: Vec<Line> = Vec::from([
			Line::from_string(String::from(""), 0),
			Line::from_string(String::from("# Bar"), 0),
			Line::from_string(String::from(""), 0),
			Line::from_string(String::from("Z Baz"), 0),
		]);

		let expected_moved = 0;
		let expected_todo = Vec::from([
			Line::from_string(String::from(""), 0),
			Line::from_string(String::from("# Bar"), 0),
			Line::from_string(String::from(""), 0),
			Line::from_string(String::from("Z Baz"), 0),
		]);

		let dir = tempdir().unwrap();

		let todo_filename = dir
			.path()
			.join("todo-X88.txt")
			.as_path()
			.display()
			.to_string();
		{
			let mut l = List::new();
			l.lines = initial_todo;
			l.to_filename(todo_filename.clone());
		}

		let done_filename = dir
			.path()
			.join("done-X88.txt")
			.as_path()
			.display()
			.to_string();

		let (moved, result) = run_archive(&todo_filename, &done_filename);
		assert_eq!(expected_moved, moved);
		assert!(_eq_vecline(result.lines, expected_todo.clone()));
		assert!(_eq_vecline(
			List::from_filename(todo_filename.clone())
				.unwrap()
				.lines,
			expected_todo
		));
	}
}

use crate::tadaitem::*;
use lazy_static::lazy_static;
use regex::Regex;
use std::fs::File;
use std::io::{BufRead, BufReader};

/// A line type — item, comment, or blank
#[derive(Debug, Eq, PartialEq)]
pub enum TadaListLineKind {
	Item,
	Comment,
	Blank,
}

/// An line in a todo list.
#[derive(Debug)]
pub struct TadaListLine {
	pub kind: TadaListLineKind,
	pub text: String,
	pub item: Option<TadaItem>,
}

/// A todo list.
#[derive(Debug)]
pub struct TadaList {
	pub lines: Vec<TadaListLine>,
}

lazy_static! {
	/// Regular expression to match lines which are entirely whitespace.
	static ref RE_LINE_BLANK: Regex = Regex::new(r"^\s*$").unwrap();
	/// Regular expression to match lines which are comments.
	static ref RE_LINE_COMMENT: Regex = Regex::new(r"^\s*#").unwrap();
}

impl TadaList {
	/// Parse a todo list from an open file.
	pub fn new_from_file(f: File) -> TadaList {
		let io = BufReader::new(f);
		let lines = io
			.lines()
			.map(|l| Self::_handle_line(l.unwrap()))
			.collect();
		TadaList { lines }
	}

	/// Helper function to convert a single line string into a TadaListLine.
	fn _handle_line(line: String) -> TadaListLine {
		if RE_LINE_BLANK.is_match(&line) {
			TadaListLine {
				text: line,
				kind: TadaListLineKind::Blank,
				item: None,
			}
		} else if RE_LINE_COMMENT.is_match(&line) {
			TadaListLine {
				text: line,
				kind: TadaListLineKind::Comment,
				item: None,
			}
		} else {
			let parsed = TadaItem::parse(&line);
			TadaListLine {
				text: line,
				kind: TadaListLineKind::Item,
				item: Some(parsed),
			}
		}
	}

	/// Get a Vec of TadaItem objects from an already-parsed file.
	pub fn items(&self) -> Vec<&TadaItem> {
		let mut items = Vec::new();
		for line in &self.lines {
			if line.kind == TadaListLineKind::Item {
				let item = line.item.as_ref().unwrap();
				items.push(item);
			}
		}
		items
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::fs::File;
	use std::io::Write;
	use tempfile::tempdir;

	#[test]
	fn test_new_from_file() {
		let dir = tempdir().unwrap();
		let file_path = dir.path().join("test-todo.txt");
		let mut f1 = File::create(&file_path).unwrap();
		writeln!(f1, "# Comment").unwrap();
		writeln!(f1, "x (A) 2000-01-01 Foo bar @baz").unwrap();
		writeln!(f1, "  ").unwrap();

		let f2 = File::open(&file_path).unwrap();

		let list = TadaList::new_from_file(f2);
		assert_eq!(3, list.lines.len());

		let line = list.lines.get(0).unwrap();
		assert_eq!(TadaListLineKind::Comment, line.kind);
		assert_eq!("# Comment", line.text);

		let line = list.lines.get(1).unwrap();
		assert_eq!(TadaListLineKind::Item, line.kind);
		assert_eq!('A', line.item.as_ref().unwrap().priority);

		let line = list.lines.get(2).unwrap();
		assert_eq!(TadaListLineKind::Blank, line.kind);
	}

	#[test]
	fn test_items() {
		let dir = tempdir().unwrap();
		let file_path = dir.path().join("test-todo.txt");
		let mut f1 = File::create(&file_path).unwrap();
		writeln!(f1, "# Comment").unwrap();
		writeln!(f1, "x (A) 2000-01-01 Foo bar @baz").unwrap();
		writeln!(f1, "  ").unwrap();

		let f2 = File::open(&file_path).unwrap();

		let list = TadaList::new_from_file(f2);
		let items = list.items();
		assert_eq!(1, items.len());

		let item = items.get(0).unwrap();
		assert_eq!('A', item.priority);
		assert!(item.creation_date.is_some());
	}
}

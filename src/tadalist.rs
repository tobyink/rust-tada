use crate::tadaitem::*;
use lazy_static::lazy_static;
use regex::Regex;
use std::fs::File;
use std::io::{BufRead, BufReader};

/// A line type — item, comment, or blank
#[derive(Debug, Eq, PartialEq)]
pub enum TadaListLineType {
	Item,
	Comment,
	Blank,
}

/// An line in a todo list.
#[derive(Debug)]
pub struct TadaListLine {
	pub line_type: TadaListLineType,
	pub text: String,
	pub item: Option<TadaItem>,
}

/// A todo list.
#[derive(Debug)]
pub struct TadaList {
	pub lines: Vec<TadaListLine>,
}

lazy_static! {
	static ref RE_LINE_COMMENT: Regex = Regex::new(r"^\s*#").unwrap();
	static ref RE_LINE_BLANK: Regex = Regex::new(r"^\s*$").unwrap();
}

impl TadaList {
	/// Parse a todo list from an open file.
	pub fn new_from_file(f: File) -> TadaList {
		let io = BufReader::new(f);
		let mut lines = Vec::new();

		for line in io.lines() {
			let tl_line = Self::_handle_line(line.unwrap());
			lines.push(tl_line);
		}

		TadaList { lines }
	}

	fn _handle_line(line: String) -> TadaListLine {
		if RE_LINE_BLANK.is_match(&line) {
			TadaListLine {
				text: line,
				line_type: TadaListLineType::Blank,
				item: None,
			}
		} else if RE_LINE_COMMENT.is_match(&line) {
			TadaListLine {
				text: line,
				line_type: TadaListLineType::Comment,
				item: None,
			}
		} else {
			let parsed = TadaItem::parse(&line);
			TadaListLine {
				text: line,
				line_type: TadaListLineType::Item,
				item: Some(parsed),
			}
		}
	}

	pub fn items(&self) -> Vec<&TadaItem> {
		let mut items = Vec::new();
		for line in &self.lines {
			if line.line_type == TadaListLineType::Item {
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
		assert_eq!(TadaListLineType::Comment, line.line_type);
		assert_eq!("# Comment", line.text);

		let line = list.lines.get(1).unwrap();
		assert_eq!(TadaListLineType::Item, line.line_type);
		assert_eq!('A', line.item.as_ref().unwrap().priority);

		let line = list.lines.get(2).unwrap();
		assert_eq!(TadaListLineType::Blank, line.line_type);
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

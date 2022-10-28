use crate::tadaitem::*;
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

impl TadaList {
	/// Parse a todo list from an open file.
	pub fn new_from_file(f: File) -> TadaList {
		let io = BufReader::new(f);
		let mut lines = Vec::new();

		let re_comment = Regex::new(r"^\s*#").unwrap();
		let re_blank = Regex::new(r"^\s*$").unwrap();

		for line in io.lines() {
			let got = line.unwrap();
			let tl_line = if re_blank.is_match(&got) {
				TadaListLine {
					text: got,
					line_type: TadaListLineType::Blank,
					item: None,
				}
			} else if re_comment.is_match(&got) {
				TadaListLine {
					text: got,
					line_type: TadaListLineType::Comment,
					item: None,
				}
			} else {
				let cloned = got.clone();
				TadaListLine {
					text: got,
					line_type: TadaListLineType::Item,
					item: Some(TadaItem::parse(&cloned)),
				}
			};

			lines.push(tl_line);
		}

		TadaList { lines }
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
	use tempfile::tempdir;
	use std::fs::File;
	use std::io::Write;

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
		
		let line0 = list.lines.get(0).unwrap();
		assert_eq!(TadaListLineType::Comment, line0.line_type);
		assert_eq!("# Comment", line0.text);
		
		let line1 = list.lines.get(1).unwrap();
		assert_eq!(TadaListLineType::Item, line1.line_type);
		assert_eq!('A', line1.item.as_ref().unwrap().priority);
		
		let line2 = list.lines.get(2).unwrap();
		assert_eq!(TadaListLineType::Blank, line2.line_type);
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
		
		let item0 = items.get(0).unwrap();
		assert_eq!('A', item0.priority);
	}
}
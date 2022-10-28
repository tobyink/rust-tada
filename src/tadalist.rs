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

//! Types related to todo list files.
//!
//! # Examples
//!
//! ```
//! use tada::list::{List,LineKind};
//!
//! let data = String::from("(A) Foo\n(B) Bar\n\n# Comment\n");
//! let list = List::from_string(data).unwrap();
//!
//! // Use list.lines to access raw lines from the list.
//! assert_eq!(LineKind::Item, list.lines[0].kind);
//! assert!(list.lines[0].item.is_some());
//! assert_eq!(LineKind::Item, list.lines[1].kind);
//! assert!(list.lines[1].item.is_some());
//! assert_eq!(LineKind::Blank, list.lines[2].kind);
//! assert!(list.lines[2].item.is_none());
//! assert_eq!(LineKind::Comment, list.lines[3].kind);
//! assert!(list.lines[3].item.is_none());
//!
//! // Use list.items() to get todo items from the list.
//! let items = list.items();
//! assert_eq!(2, items.len());
//! ```

use crate::item::{Item, Urgency};
use lazy_static::lazy_static;
use path_absolutize::*;
use regex::Regex;
use reqwest::blocking::Client;
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader, Error, Write};
use std::path::Path;
use url::Url;

lazy_static! {
	/// Regular expression to match lines which are entirely whitespace.
	static ref RE_LINE_BLANK: Regex = Regex::new(r"^\s*$").unwrap();
	/// Regular expression to match lines which are comments.
	static ref RE_LINE_COMMENT: Regex = Regex::new(r"^\s*#").unwrap();
}

/// A line type.
#[derive(Debug, Eq, PartialEq, Clone)]
pub enum LineKind {
	/// A line representing a task.
	Item,
	/// A line which starts with a `#`.
	Comment,
	/// An empty line.
	Blank,
}

/// An line in a todo list.
#[derive(Debug, Clone)]
pub struct Line {
	pub kind: LineKind,
	pub text: String,
	pub item: Option<Item>,
	pub num: usize,
}

impl Line {
	pub fn new_blank() -> Line {
		Line {
			kind: LineKind::Blank,
			text: String::new(),
			item: None,
			num: 0,
		}
	}

	/// Create a Line struct by parsing a string.
	pub fn from_string(text: String, num: usize) -> Line {
		let item = None;
		if RE_LINE_BLANK.is_match(&text) {
			let kind = LineKind::Blank;
			Line {
				text,
				kind,
				item,
				num,
			}
		} else if RE_LINE_COMMENT.is_match(&text) {
			let kind = LineKind::Comment;
			Line {
				text,
				kind,
				item,
				num,
			}
		} else {
			let kind = LineKind::Item;
			let mut item = Item::parse(&text);
			item.set_line_number(num);
			Line {
				text,
				kind,
				item: Some(item),
				num,
			}
		}
	}

	/// Wrap an Item struct to be a Line.
	pub fn from_item(item: Item) -> Line {
		Line {
			kind: LineKind::Item,
			text: format!("{}", item),
			item: Some(item),
			num: 0,
		}
	}

	/// Create a version of this line but representing a completed task.
	pub fn but_done(&self, include_date: bool) -> Line {
		match self.kind {
			LineKind::Item => {
				let item = self.clone().item.unwrap();
				Line::from_item(item.but_done(include_date))
			}
			_ => self.clone(),
		}
	}

	/// Create a version of this line but representing a pulled task.
	pub fn but_pull(&self, new_urgency: Urgency) -> Line {
		match self.kind {
			LineKind::Item => {
				let item = self.clone().item.unwrap();
				Line::from_item(item.but_pull(new_urgency))
			}
			_ => self.clone(),
		}
	}
}

impl Default for Line {
	fn default() -> Self {
		Self::new_blank()
	}
}

/// A todo list.
#[derive(Debug)]
pub struct List {
	pub path: Option<String>,
	pub lines: Vec<Line>,
}

impl List {
	pub fn new() -> Self {
		Self {
			path: None,
			lines: Vec::new(),
		}
	}

	fn _handle_url(u: String) -> Url {
		Url::parse(&u).unwrap_or_else(|_| {
			let p = Path::new(&u);
			Url::from_file_path(p.absolutize().unwrap().to_str().unwrap())
				.unwrap()
		})
	}

	pub fn from_items(lines: Vec<&Item>) -> Self {
		let mut list = List::new();
		for l in lines {
			list.lines.push(Line::from_item(l.clone()));
		}
		list
	}

	/// Parse a todo list from a URL.
	pub fn from_url(u: String) -> Result<Self, Error> {
		let url = Self::_handle_url(u);
		match url.scheme() {
			"file" => Self::from_filename(
				url.to_file_path()
					.unwrap()
					.into_os_string()
					.into_string()
					.unwrap(),
			),
			"http" | "https" => Self::from_http(url),
			_ => panic!("non-file URL: {:?}", url),
		}
	}

	/// Parse a todo list from a filename.
	pub fn from_filename(path: String) -> Result<Self, Error> {
		let file = File::open(&path)?;
		let mut list = Self::from_file(file)?;
		list.path = Some(path);
		Ok(list)
	}

	/// Parse a todo list from an open file.
	pub fn from_file(f: File) -> Result<Self, Error> {
		let mut count = 0;
		let io = BufReader::new(f);
		let lines = io
			.lines()
			.map(|l| {
				count += 1;
				Line::from_string(l.unwrap(), count)
			})
			.collect();
		let list = List { path: None, lines };
		Ok(list)
	}

	/// Parse a todo list from a string.
	pub fn from_string(s: String) -> Result<Self, Error> {
		let mut count = 0;
		let lines = s
			.lines()
			.map(|l| {
				count += 1;
				Line::from_string(l.to_string(), count)
			})
			.collect();
		let list = List { path: None, lines };
		Ok(list)
	}

	/// Read a todo list over HTTP.
	pub fn from_http(url: Url) -> Result<Self, Error> {
		let client = Client::new();
		let mut request = client.get(url);
		if let Ok(x) = env::var("TADA_HTTP_USER_AGENT") {
			request = request.header(reqwest::header::USER_AGENT, x);
		}
		if let Ok(x) = env::var("TADA_HTTP_AUTHORIZATION") {
			request = request.header(reqwest::header::AUTHORIZATION, x.clone());
			request = request.header("X-Tada-Authorization", x);
		}
		if let Ok(x) = env::var("TADA_HTTP_FROM") {
			request = request.header(reqwest::header::FROM, x);
		}
		let response = request.send().unwrap();
		if response.status().is_success() {
			return Self::from_string(response.text().unwrap());
		}
		Err(Error::new(
			std::io::ErrorKind::Other,
			format!("HTTP response: {}", response.status()),
		))
	}

	// Save a todo list to a URL.
	pub fn to_url(&self, u: String) {
		let url = Self::_handle_url(u);
		match url.scheme() {
			"file" => {
				self.to_filename(
					url.to_file_path()
						.unwrap()
						.into_os_string()
						.into_string()
						.unwrap(),
				);
			}
			"http" | "https" => {
				self.to_http(url);
			}
			_ => panic!("non-file URL"),
		}
	}

	/// Save a todo list to a filename.
	pub fn to_filename(&self, path: String) {
		let file = match File::create(&path) {
			Err(why) => panic!("Couldn't create file {}: {}", path, why),
			Ok(file) => file,
		};
		self.to_file(file);
	}

	/// Save a todo list to a file.
	pub fn to_file(&self, mut f: File) {
		if let Err(why) = f.write_all(self.serialize().as_bytes()) {
			panic!("Couldn't write to file: {}", why);
		};
	}

	/// Save a todo list using an HTTP PUT request.
	pub fn to_http(&self, url: Url) {
		let client = Client::new();
		let mut request = client.put(url);
		if let Ok(x) = env::var("TADA_HTTP_USER_AGENT") {
			request = request.header(reqwest::header::USER_AGENT, x);
		}
		if let Ok(x) = env::var("TADA_HTTP_AUTHORIZATION") {
			request = request.header(reqwest::header::AUTHORIZATION, x.clone());
			request = request.header("X-Tada-Authorization", x);
		}
		if let Ok(x) = env::var("TADA_HTTP_FROM") {
			request = request.header(reqwest::header::FROM, x);
		}
		request = request.header(reqwest::header::CONTENT_TYPE, "text/plain");
		println!("{:?}", request);
		let response = request.body(self.serialize()).send().unwrap();
		println!("{:?}", response);
		if !response.status().is_success() {
			panic!("HTTP response: {}", response.status());
		}
	}

	/// Serialize a todo list as a string.
	pub fn serialize(&self) -> String {
		self.lines
			.iter()
			.map(|l| l.text.clone() + "\n")
			.collect::<String>()
	}

	/// Appends some lines to a todo list, given its filename.
	pub fn append_lines_to_url(u: String, lines: Vec<&Line>) {
		let url = Self::_handle_url(u);

		// XXX: If the URL is a local file path, shortcut this using a simple file append.
		let mut list = Self::from_url(url.to_string()).unwrap_or_else(|_| {
			panic!("Could not open list {} to append to", url)
		});
		for l in lines {
			list.lines.push(l.clone());
		}
		list.to_url(url.to_string());
	}

	/// Get a Vec<&Item> from an already-parsed file.
	pub fn items(&self) -> Vec<&Item> {
		let iter = self.lines.iter();
		iter.filter(|l| l.kind == LineKind::Item)
			.map(|l| l.item.as_ref().unwrap())
			.collect()
	}

	/// Count the items in the list.
	pub fn count_items(&self) -> usize {
		self.lines
			.iter()
			.filter(|l| l.kind == LineKind::Item)
			.count()
	}

	/// Count the blank/comment lines in the list.
	pub fn count_blank(&self) -> usize {
		self.lines
			.iter()
			.filter(|l| l.kind != LineKind::Item)
			.count()
	}

	/// Count the completed items in the list.
	pub fn count_completed(&self) -> usize {
		self.lines
			.iter()
			.filter(|l| {
				l.kind == LineKind::Item && l.item.clone().unwrap().completion()
			})
			.count()
	}

	/// Clone the list, but removing blank lines and comments, and optionally sort it.
	pub fn but_tidy(&self, sort_order: &crate::action::SortOrder) -> Self {
		let mut new_list = Self::new();
		for item in sort_order.sort_items(self.items()).into_iter() {
			new_list
				.lines
				.push(Line::from_item(item.clone()));
		}
		new_list
	}
}

impl Default for List {
	fn default() -> Self {
		Self::new()
	}
}

#[cfg(test)]
mod tests_list {
	use super::*;
	use std::fs::File;
	use std::io::Write;
	use tempfile::tempdir;

	#[test]
	fn test_from_file() {
		let dir = tempdir().unwrap();
		let file_path = dir.path().join("test-todo.txt");
		let mut f1 = File::create(&file_path).unwrap();
		writeln!(f1, "# Comment").unwrap();
		writeln!(f1, "x (A) 2000-01-01 Foo bar @baz").unwrap();
		writeln!(f1, "  ").unwrap();

		let f2 = File::open(&file_path).unwrap();

		let list = List::from_file(f2).unwrap();
		assert_eq!(3, list.lines.len());

		let line = list.lines.get(0).unwrap();
		assert_eq!(LineKind::Comment, line.kind);
		assert_eq!("# Comment", line.text);

		let line = list.lines.get(1).unwrap();
		assert_eq!(LineKind::Item, line.kind);
		assert_eq!('A', line.item.as_ref().unwrap().priority());

		let line = list.lines.get(2).unwrap();
		assert_eq!(LineKind::Blank, line.kind);
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

		let list = List::from_file(f2).unwrap();
		let items = list.items();
		assert_eq!(1, items.len());

		let item = items.get(0).unwrap();
		assert_eq!('A', item.priority());
		assert!(item.creation_date().is_some());
	}
}

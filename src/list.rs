use crate::item::Item;
use lazy_static::lazy_static;
use regex::Regex;
use reqwest::blocking::Client;
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader, Error, Write};
use url::Url;

/// A line type — item, comment, or blank
#[derive(Debug, Eq, PartialEq, Clone)]
pub enum LineKind {
	Item,
	Comment,
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

/// A todo list.
#[derive(Debug)]
pub struct List {
	pub path: Option<String>,
	pub lines: Vec<Line>,
}

lazy_static! {
	/// Regular expression to match lines which are entirely whitespace.
	static ref RE_LINE_BLANK: Regex = Regex::new(r"^\s*$").unwrap();
	/// Regular expression to match lines which are comments.
	static ref RE_LINE_COMMENT: Regex = Regex::new(r"^\s*#").unwrap();
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
}

impl List {
	pub fn new() -> List {
		List {
			path: None,
			lines: Vec::new(),
		}
	}

	fn _handle_url(u: String) -> Url {
		Url::parse(&u).unwrap_or_else(|_| Url::from_file_path(u).unwrap())
	}

	/// Parse a todo list from a URL.
	pub fn from_url(u: String) -> Result<List, Error> {
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
			"sftp" => {
				let mut host = String::from(url.host_str().unwrap());
				let username = String::from(url.username());
				if !username.is_empty() {
					host = username + "@" + &host;
				}
				if let Some(password) = url.port() {
					host = "ssh://".to_owned()
						+ &host + ":" + &password.to_string();
				}
				let mut path = String::from(url.path());
				if path.starts_with("/~/") {
					path = String::from(path.get(3..).unwrap());
				}
				List::from_sftp(host, path)
			}
			_ => panic!("non-file URL: {:?}", url),
		}
	}

	/// Parse a todo list from a filename.
	pub fn from_filename(path: String) -> Result<List, Error> {
		let file = File::open(&path)?;
		let mut list = Self::from_file(file)?;
		list.path = Some(path);
		Ok(list)
	}

	/// Parse a todo list from an open file.
	pub fn from_file(f: File) -> Result<List, Error> {
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
	pub fn from_string(s: String) -> Result<List, Error> {
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
	pub fn from_http(url: Url) -> Result<List, Error> {
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

	/// Read a todo list over SFTP (not implemented yet).
	pub fn from_sftp(_host: String, _path: String) -> Result<List, Error> {
		panic!("SFTP not implemented yet")
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

	/// Get a Vec of Item objects from an already-parsed file.
	pub fn items(&self) -> Vec<&Item> {
		let iter = self.lines.iter();
		iter.filter(|l| l.kind == LineKind::Item)
			.map(|l| l.item.as_ref().unwrap())
			.collect()
	}
}

impl Default for Line {
	fn default() -> Self {
		Self::new_blank()
	}
}

impl Default for List {
	fn default() -> Self {
		Self::new()
	}
}

#[cfg(test)]
mod tests {
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

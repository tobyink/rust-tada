use chrono::NaiveDate;
use lazy_static::lazy_static;
use regex::Regex;
use std::fmt;

/// An item in a todo list.
pub struct TadaItem {
	pub completion: bool,
	pub priority: char,
	pub completion_date: Option<NaiveDate>,
	pub creation_date: Option<NaiveDate>,
	pub description: String,
}

impl fmt::Debug for TadaItem {
	/// Debugging output; used for format!("{:?}")
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_struct("Item")
			.field("completion", &self.completion)
			.field("priority", &self.priority)
			.field("completion_date", &self.completion_date)
			.field("creation_date", &self.creation_date)
			.field("description", &self.description)
			.finish()
	}
}

impl fmt::Display for TadaItem {
	/// File-ready output; used for format!("{}")
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let mut r: String = String::new();

		if self.completion {
			r.push_str("x ");
		}

		if self.priority != '\0' {
			let paren = format!("({}) ", self.priority);
			r.push_str(&paren);
		}

		if self.completion && self.completion_date.is_some() {
			let date1 = self
				.completion_date
				.unwrap()
				.format("%Y-%m-%d ")
				.to_string();
			r.push_str(&date1);
		}

		if self.creation_date.is_some() {
			let date2 = self
				.creation_date
				.unwrap()
				.format("%Y-%m-%d ")
				.to_string();
			r.push_str(&date2);
		}

		r.push_str(&self.description);

		write!(f, "{}", r)
	}
}

lazy_static! {
	/// Regular expression to capture the parts of a tada list line.
	static ref RE_TADA_ITEM: Regex = Regex::new(
		r"^(x )?(\([A-Z]\) )?(\d{4}-\d{2}-\d{2} )?(\d{4}-\d{2}-\d{2} )?(.+)$"
	)
	.unwrap();
}

impl TadaItem {
	/// Parse an item from a line of text.
	///
	/// Assumes the [todo.txt](https://github.com/todotxt/todo.txt) format.
	pub fn parse(text: &str) -> TadaItem {
		let caps = RE_TADA_ITEM.captures(text).unwrap();

		TadaItem {
			completion: caps.get(1).is_some(),
			priority: match caps.get(2) {
				Some(p) => p.as_str().chars().nth(1).unwrap(),
				None => '\0',
			},
			completion_date: if caps.get(3).is_some() && caps.get(4).is_some() {
				let cap3 = caps.get(3).unwrap().as_str();
				NaiveDate::parse_from_str(cap3, "%Y-%m-%d ").ok()
			} else {
				None
			},
			creation_date: if caps.get(3).is_some() && caps.get(4).is_some() {
				let cap4 = caps.get(4).unwrap().as_str();
				NaiveDate::parse_from_str(cap4, "%Y-%m-%d ").ok()
			} else if caps.get(3).is_some() {
				let cap3 = caps.get(3).unwrap().as_str();
				NaiveDate::parse_from_str(cap3, "%Y-%m-%d ").ok()
			} else {
				None
			},
			description: match caps.get(5) {
				Some(m) => String::from(m.as_str()),
				None => String::from(""),
			},
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use chrono::NaiveDate;

	#[test]
	fn test_debug() {
		let i = TadaItem {
			completion: false,
			priority: '\0',
			completion_date: None,
			creation_date: None,
			description: "foo bar baz".to_string(),
		};
		let dbug = format!("{:?}", i);
		assert!(dbug.len() > 1);
	}

	#[test]
	fn test_display() {
		let i = TadaItem {
			completion: false,
			priority: '\0',
			completion_date: None,
			creation_date: None,
			description: "foo bar baz".to_string(),
		};

		assert_eq!("foo bar baz", format!("{}", i));

		let i = TadaItem {
			completion: true,
			priority: 'B',
			completion_date: Some(NaiveDate::from_ymd(2010, 1, 1)),
			creation_date: Some(NaiveDate::from_ymd(2000, 12, 31)),
			description: "foo bar baz".to_string(),
		};

		assert_eq!("x (B) 2010-01-01 2000-12-31 foo bar baz", format!("{}", i));
	}

	#[test]
	fn test_parse() {
		let i = TadaItem::parse("x (B) 2010-01-01 2000-12-31 foo bar baz");

		assert_eq!(true, i.completion);
		assert_eq!('B', i.priority);
		assert_eq!(NaiveDate::from_ymd(2010, 1, 1), i.completion_date.unwrap());
		assert_eq!(NaiveDate::from_ymd(2000, 12, 31), i.creation_date.unwrap());
		assert_eq!("foo bar baz".to_string(), i.description);

		let i = TadaItem::parse("2010-01-01 (A) foo bar baz");

		assert!(!i.completion);
		assert_eq!('\0', i.priority);
		assert!(!i.completion_date.is_some());
		assert_eq!(NaiveDate::from_ymd(2010, 1, 1), i.creation_date.unwrap());
		assert_eq!("(A) foo bar baz".to_string(), i.description);
	}
}

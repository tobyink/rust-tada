use chrono::NaiveDate;
use regex::Regex;
use std::fmt;

/// An item in a todo list.
pub struct TadaItem {
	pub completion: bool,
	pub has_priority: bool,
	pub priority: char,
	pub has_completion_date: bool,
	pub completion_date: NaiveDate,
	pub has_creation_date: bool,
	pub creation_date: NaiveDate,
	pub description: String,
}

impl fmt::Debug for TadaItem {
	/// Debugging output; used for format!("{:?}")
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_struct("Item")
			.field("completion", &self.completion)
			.field("priority", &self.priority)
			.field("completion_date", &self.completion_date)
			.field("description", &self.description)
			.finish()
	}
}

impl fmt::Display for TadaItem {
	/// File-ready output; used for format!("{}")
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let mut r: String = "".to_string();

		if self.completion {
			r.push_str("x ");
		}

		if self.has_priority {
			let paren = format!("({}) ", self.priority);
			r.push_str(&paren);
		}

		if self.completion && self.has_completion_date {
			let date1 = self.completion_date.format("%Y-%m-%d ").to_string();
			r.push_str(&date1);
		}

		if self.has_creation_date {
			let date2 = self.creation_date.format("%Y-%m-%d ").to_string();
			r.push_str(&date2);
		}

		r.push_str(&self.description);

		write!(f, "{}", r)
	}
}

impl TadaItem {
	/// Parse an item from a line of text.
	///
	/// Assumes the [todo.txt](https://github.com/todotxt/todo.txt) format.
	pub fn parse(text: &str) -> TadaItem {
		let re = Regex::new(r"^(x )?(\([A-Z]\) )?(\d{4}-\d{2}-\d{2} )?(\d{4}-\d{2}-\d{2} )?(.+)$")
			.unwrap();
		let caps = re.captures(text).unwrap();

		let mut r = TadaItem {
			completion: false,
			has_priority: false,
			priority: '\0',
			has_completion_date: false,
			completion_date: NaiveDate::from_ymd(0, 1, 1),
			has_creation_date: false,
			creation_date: NaiveDate::from_ymd(0, 1, 1),
			description: caps.get(5).map_or("", |m| m.as_str()).to_string(),
		};

		if caps.get(1) != None {
			r.completion = true;
		}

		if caps.get(2) != None {
			let matched = caps.get(2).unwrap().as_str();
			r.priority = matched.chars().nth(1).unwrap();
			r.has_priority = true;
		}

		if caps.get(3) != None {
			if caps.get(4) != None {
				r.has_completion_date = true;
				r.has_creation_date = true;
				r.completion_date =
					NaiveDate::parse_from_str(caps.get(3).unwrap().as_str(), "%Y-%m-%d ").unwrap();
				r.creation_date =
					NaiveDate::parse_from_str(caps.get(4).unwrap().as_str(), "%Y-%m-%d ").unwrap();
			} else {
				r.has_creation_date = true;
				r.creation_date =
					NaiveDate::parse_from_str(caps.get(3).unwrap().as_str(), "%Y-%m-%d ").unwrap();
			}
		}

		r
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
			has_priority: false,
			priority: '\0',
			has_completion_date: false,
			completion_date: NaiveDate::from_ymd(0, 1, 1),
			has_creation_date: false,
			creation_date: NaiveDate::from_ymd(0, 1, 1),
			description: "foo bar baz".to_string(),
		};
		let dbug = format!("{:?}", i);
		assert!(dbug.len() > 1);
	}

	#[test]
	fn test_display() {
		let i = TadaItem {
			completion: false,
			has_priority: false,
			priority: '\0',
			has_completion_date: false,
			completion_date: NaiveDate::from_ymd(0, 1, 1),
			has_creation_date: false,
			creation_date: NaiveDate::from_ymd(0, 1, 1),
			description: "foo bar baz".to_string(),
		};

		assert_eq!("foo bar baz", format!("{}", i));

		let j = TadaItem {
			completion: true,
			has_priority: true,
			priority: 'B',
			has_completion_date: true,
			completion_date: NaiveDate::from_ymd(2010, 1, 1),
			has_creation_date: true,
			creation_date: NaiveDate::from_ymd(2000, 12, 31),
			description: "foo bar baz".to_string(),
		};

		assert_eq!("x (B) 2010-01-01 2000-12-31 foo bar baz", format!("{}", j));
	}

	#[test]
	fn test_parse() {
		let i = TadaItem::parse("x (B) 2010-01-01 2000-12-31 foo bar baz");

		assert_eq!(true, i.completion);
		assert_eq!(true, i.has_priority);
		assert_eq!('B', i.priority);
		assert_eq!(true, i.has_completion_date);
		assert_eq!(NaiveDate::from_ymd(2010, 1, 1), i.completion_date);
		assert_eq!(true, i.has_creation_date);
		assert_eq!(NaiveDate::from_ymd(2000, 12, 31), i.creation_date);
		assert_eq!("foo bar baz".to_string(), i.description);
	}
}

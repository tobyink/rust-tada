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

/// Syntax for a tada item line.
const RE_TADA_ITEM: &str = r"^(x )?(\([A-Z]\) )?(\d{4}-\d{2}-\d{2} )?(\d{4}-\d{2}-\d{2} )?(.+)$";

impl TadaItem {
	/// Parse an item from a line of text.
	///
	/// Assumes the [todo.txt](https://github.com/todotxt/todo.txt) format.
	pub fn parse(text: &str) -> TadaItem {
		let caps = Regex::new(RE_TADA_ITEM).unwrap().captures(text).unwrap();

		let descr = caps.get(5).map_or("", |m| m.as_str());
		let mut r = TadaItem {
			completion: caps.get(1).is_some(),
			has_priority: caps.get(2).is_some(),
			priority: match caps.get(2) {
				Some(p) => p.as_str().chars().nth(1).unwrap(),
				None => '\0',
			},
			has_completion_date: false,
			completion_date: NaiveDate::from_ymd(0, 1, 1),
			has_creation_date: false,
			creation_date: NaiveDate::from_ymd(0, 1, 1),
			description: String::from(descr),
		};

		if caps.get(3).is_some() {
			let cap3 = caps.get(3).unwrap();
			// If cap3 and cap4 are both set, then they are the completion date and creation date.
			// If only cap3 is set, it's the creation date.
			if caps.get(4).is_some() {
				let cap4 = caps.get(4).unwrap();
				r.has_completion_date = true;
				r.has_creation_date = true;
				r.completion_date = NaiveDate::parse_from_str(cap3.as_str(), "%Y-%m-%d ").unwrap();
				r.creation_date = NaiveDate::parse_from_str(cap4.as_str(), "%Y-%m-%d ").unwrap();
			} else {
				r.has_creation_date = true;
				r.creation_date = NaiveDate::parse_from_str(cap3.as_str(), "%Y-%m-%d ").unwrap();
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

		let j = TadaItem::parse("2010-01-01 (A) foo bar baz");

		assert_eq!(false, j.completion);
		assert_eq!(false, j.has_priority);
		assert_eq!(false, j.has_completion_date);
		assert_eq!(true, j.has_creation_date);
		assert_eq!(NaiveDate::from_ymd(2010, 1, 1), j.creation_date);
		assert_eq!("(A) foo bar baz".to_string(), j.description);
	}
}

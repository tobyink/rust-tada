use chrono::{Datelike, Duration, NaiveDate, Utc, Weekday};
use lazy_static::lazy_static;
use regex::Regex;
use std::fmt;

/// An item in a todo list.
pub struct Item {
	pub completion: bool,
	pub priority: char,
	pub completion_date: Option<NaiveDate>,
	pub creation_date: Option<NaiveDate>,
	pub description: String,
}

/// Seven levels of urgency are defined.
#[allow(dead_code)]
pub enum Urgency {
	Overdue,
	Today,
	Soon,
	ThisWeek,
	NextWeek,
	NextMonth,
	Later,
}

/// Three sizes are defined.
#[allow(dead_code)]
pub enum TshirtSize {
	Small,
	Medium,
	Large,
}

impl fmt::Debug for Item {
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

impl fmt::Display for Item {
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
			let date = self
				.completion_date
				.unwrap()
				.format("%Y-%m-%d ")
				.to_string();
			r.push_str(&date);
		}

		if self.creation_date.is_some() {
			let date = self
				.creation_date
				.unwrap()
				.format("%Y-%m-%d ")
				.to_string();
			r.push_str(&date);
		}

		r.push_str(&self.description);

		write!(f, "{}", r)
	}
}

lazy_static! {
	/// Regular expression to capture the parts of a tada list line.
	static ref RE_TADA_ITEM: Regex = Regex::new(r##"(?x)
		^                               # start
		( x \s+ )?                      # optional "x"
		( [(] [A-Z] [)] \s+ )?          # optional priority letter
		( \d{4} - \d{2} - \d{2} \s+ )?  # optional date
		( \d{4} - \d{2} - \d{2} \s+ )?  # optional date
		( .+ )                          # description
		$                               # end
	"##)
	.unwrap();
}

impl Item {
	/// Parse an item from a line of text.
	///
	/// Assumes the [todo.txt](https://github.com/todotxt/todo.txt) format.
	pub fn parse(text: &str) -> Item {
		let caps = RE_TADA_ITEM.captures(text).unwrap();

		Item {
			completion: caps.get(1).is_some(),
			priority: match caps.get(2) {
				Some(p) => p.as_str().chars().nth(1).unwrap(),
				None => '\0',
			},
			completion_date: if caps.get(3).is_some() && caps.get(4).is_some() {
				let cap3 = caps.get(3).unwrap().as_str();
				NaiveDate::parse_from_str(cap3.trim(), "%Y-%m-%d").ok()
			} else {
				None
			},
			creation_date: if caps.get(3).is_some() && caps.get(4).is_some() {
				let cap4 = caps.get(4).unwrap().as_str();
				NaiveDate::parse_from_str(cap4.trim(), "%Y-%m-%d").ok()
			} else if caps.get(3).is_some() {
				let cap3 = caps.get(3).unwrap().as_str();
				NaiveDate::parse_from_str(cap3.trim(), "%Y-%m-%d").ok()
			} else {
				None
			},
			description: match caps.get(5) {
				Some(m) => String::from(m.as_str().trim()),
				None => String::from(""),
			},
		}
	}

	#[allow(dead_code)]
	fn _last_day_of_next_month(date: &NaiveDate) -> NaiveDate {
		match date.month() {
			11 => NaiveDate::from_ymd_opt(date.year() + 1, 1, 1),
			12 => NaiveDate::from_ymd_opt(date.year() + 1, 2, 1),
			_ => NaiveDate::from_ymd_opt(date.year(), date.month() + 2, 1),
		}
		.unwrap()
		.pred()
	}

	/// Classify how urgent this task is.
	#[allow(dead_code)]
	fn urgency(&self) -> Option<Urgency> {
		let _date_today = Utc::now().date_naive();
		let _date_soon = _date_today + Duration::days(2);
		let _date_weekend = _date_today.week(Weekday::Mon).last_day();
		let _date_next_weekend = _date_weekend + Duration::days(7);
		let _date_next_month = Self::_last_day_of_next_month(&_date_today);

		let due = match self.due_date() {
			Some(d) => d,
			None => return None,
		};

		if due < _date_today {
			Some(Urgency::Overdue)
		} else if due == _date_today {
			Some(Urgency::Today)
		} else if due <= _date_soon {
			Some(Urgency::Soon)
		} else if due <= _date_weekend {
			Some(Urgency::ThisWeek)
		} else if due <= _date_next_weekend {
			Some(Urgency::NextWeek)
		} else if due <= _date_next_month {
			Some(Urgency::NextMonth)
		} else {
			Some(Urgency::Later)
		}
	}

	/// Return the date when this task is due by.
	///
	/// Not implemented yet - needs tag support to work.
	#[allow(dead_code)]
	fn due_date(&self) -> Option<NaiveDate> {
		None
	}

	/// Return the importance of this task.
	///
	/// Basically the same as priority, except all letters after E
	/// are treated as being the same as E. Returns None for \0.
	#[allow(dead_code)]
	fn importance(&self) -> Option<char> {
		let p = self.priority;
		match p {
			'\0' => None,
			'A' | 'B' | 'C' | 'D' => Some(p),
			_ => Some('E'),
		}
	}

	/// Return the size of this task.
	///
	/// Not implemented yet - needs tag support to work.
	#[allow(dead_code)]
	fn tshirt_size(&self) -> Option<TshirtSize> {
		None
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use chrono::NaiveDate;

	#[test]
	fn test_debug() {
		let i = Item {
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
		let i = Item {
			completion: false,
			priority: '\0',
			completion_date: None,
			creation_date: None,
			description: "foo bar baz".to_string(),
		};

		assert_eq!("foo bar baz", format!("{}", i));

		let i = Item {
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
		let i = Item::parse("x (B) 2010-01-01 2000-12-31 foo bar baz");

		assert_eq!(true, i.completion);
		assert_eq!('B', i.priority);
		assert_eq!(NaiveDate::from_ymd(2010, 1, 1), i.completion_date.unwrap());
		assert_eq!(NaiveDate::from_ymd(2000, 12, 31), i.creation_date.unwrap());
		assert_eq!("foo bar baz".to_string(), i.description);
		assert!(i.urgency().is_none());
		assert_eq!('B', i.importance().unwrap());
		assert!(i.tshirt_size().is_none());

		let i = Item::parse("2010-01-01 (A) foo bar baz");

		assert!(!i.completion);
		assert_eq!('\0', i.priority);
		assert!(!i.completion_date.is_some());
		assert_eq!(NaiveDate::from_ymd(2010, 1, 1), i.creation_date.unwrap());
		assert_eq!("(A) foo bar baz".to_string(), i.description);
		assert!(i.urgency().is_none());
		assert!(i.importance().is_none());
		assert!(i.tshirt_size().is_none());
	}
}

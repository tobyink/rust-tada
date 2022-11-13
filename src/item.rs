use chrono::{Datelike, Duration, NaiveDate, Utc, Weekday};
use console::Style;
use lazy_static::lazy_static;
use regex::Regex;
use std::cell::Cell;
use std::collections::HashMap;
use std::fmt;
use substring::Substring;

pub struct ItemFormatConfig {
	pub width: usize,
	pub colour: bool,
	pub with_creation_date: bool,
	pub with_completion_date: bool,
	pub with_newline: bool,
}

/// An item in a todo list.
pub struct Item {
	pub completion: bool,
	pub priority: char,
	pub completion_date: Option<NaiveDate>,
	pub creation_date: Option<NaiveDate>,
	pub description: String,
	// Cell<Option<Option<T>>> seems to be the best pattern for
	// implementing Moose-like lazy builders. Kind of an ugly
	// type declaration though. :(
	_importance: Cell<Option<Option<char>>>,
	_due_date: Cell<Option<Option<NaiveDate>>>,
	_urgency: Cell<Option<Option<Urgency>>>,
	_tshirt_size: Cell<Option<Option<TshirtSize>>>,
	//_tags: Cell<Option<Vec<String>>>,
	//_contexts: Cell<Option<Vec<String>>>,
	//_kv: Cell<Option<HashMap<String, String>>>,
}

/// Seven levels of urgency are defined.
#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord, Hash)]
pub enum Urgency {
	Overdue,
	Today,
	Soon,
	ThisWeek,
	NextWeek,
	NextMonth,
	Later,
}

pub static URGENCIES: [Urgency; 7] = [
	Urgency::Overdue,
	Urgency::Today,
	Urgency::Soon,
	Urgency::ThisWeek,
	Urgency::NextWeek,
	Urgency::NextMonth,
	Urgency::Later,
];

/// Three sizes are defined.
#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord, Hash)]
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
		( x \s+ )?                      # capture: optional "x"
		( [(] [A-Z] [)] \s+ )?          # capture: optional priority letter
		( \d{4} - \d{2} - \d{2} \s+ )?  # capture: optional date
		( \d{4} - \d{2} - \d{2} \s+ )?  # capture: optional date
		( .+ )                          # capture: description
		$                               # end
	"##)
	.unwrap();

	/// Regular expression to find key-value tags within a description.
	static ref RE_KV: Regex = Regex::new(r##"(?x)
		([^\s:]+)                       # capture: key
		:                               # colon
		([^\s:]+)                       # capture: value
	"##)
	.unwrap();

	/// Regular expression to find tags within a description.
	static ref RE_TAG: Regex = Regex::new(r##"(?x)
		(?:^|\s)                        # whitespace or start of string
		[+]                             # plus sign
		(\S+)                           # capture: tag
	"##)
	.unwrap();

	/// Regular expression to find contexts within a description.
	static ref RE_CONTEXT: Regex = Regex::new(r##"(?x)
		(?:^|\s)                        # whitespace or start of string
		[@]                             # at sign
		(\S+)                           # capture: context
	"##)
	.unwrap();

	/// Regular expression to match contexts indicating a small tshirt size.
	static ref RE_SMALL: Regex  = Regex::new("(?i)^X*S$").unwrap();

	/// Regular expression to match contexts indicating a medium tshirt size.
	static ref RE_MEDIUM: Regex = Regex::new("(?i)^X*M$").unwrap();

	/// Regular expression to match contexts indicating a large tshirt size.
	static ref RE_LARGE: Regex  = Regex::new("(?i)^X*L$").unwrap();

	/// Constant for today's date.
	///
	/// These date constants are evaluated once to ensure predictable behaviour
	/// when the application is run at midnight.
	///
	/// This may cause issues later on if we want to run a persistent tadalist
	/// process.
	static ref DATE_TODAY: NaiveDate = Utc::now().date_naive();

	/// Constant representing "soon".
	///
	/// Tomorrow or overmorrow.
	static ref DATE_SOON: NaiveDate = Utc::now().date_naive() + Duration::days(2);

	/// Constant representing the end of this week.
	///
	/// Weeks end on Sunday.
	static ref DATE_WEEKEND: NaiveDate = Utc::now().date_naive().week(Weekday::Mon).last_day();

	/// Constant representing the end of next week.
	static ref DATE_NEXT_WEEKEND: NaiveDate = Utc::now().date_naive().week(Weekday::Mon).last_day() + Duration::days(7);

	/// Constant representing the end of next month.
	///
	/// Who cares when *this* month ends?!
	static ref DATE_NEXT_MONTH: NaiveDate = {
		let date = Utc::now().date_naive();
		match date.month() {
			11 => NaiveDate::from_ymd_opt(date.year() + 1, 1, 1),
			12 => NaiveDate::from_ymd_opt(date.year() + 1, 2, 1),
			_ => NaiveDate::from_ymd_opt(date.year(), date.month() + 2, 1),
		}
		.unwrap()
		.pred()
	};
}

impl Item {
	pub fn new() -> Item {
		Item {
			completion: false,
			priority: '\0',
			completion_date: None,
			creation_date: None,
			description: String::new(),
			_importance: Cell::new(None),
			_due_date: Cell::new(None),
			_urgency: Cell::new(None),
			_tshirt_size: Cell::new(None),
			//_tags: Cell::new(None),
			//_contexts: Cell::new(None),
			//_kv: Cell::new(None),
		}
	}

	/// Parse an item from a line of text.
	///
	/// Assumes the [todo.txt](https://github.com/todotxt/todo.txt) format.
	pub fn parse(text: &str) -> Item {
		let caps = RE_TADA_ITEM.captures(text).unwrap();
		let blank = Self::new();

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
			..blank
		}
	}

	/// Return the importance of this task.
	///
	/// Basically the same as priority, except all letters after E
	/// are treated as being the same as E. Returns None for \0.
	pub fn importance(&self) -> Option<char> {
		let cell = &self._importance;
		if cell.get().is_none() {
			cell.set(Some(self._build_importance()));
		}
		cell.get().unwrap()
	}

	fn _build_importance(&self) -> Option<char> {
		let priority = self.priority;
		match priority {
			'\0' => None,
			'A' | 'B' | 'C' | 'D' => Some(priority),
			_ => Some('E'),
		}
	}

	/// Return the date when this task is due by.
	pub fn due_date(&self) -> Option<NaiveDate> {
		let cell = &self._due_date;
		if cell.get().is_none() {
			cell.set(Some(self._build_due_date()));
		}
		cell.get().unwrap()
	}

	fn _build_due_date(&self) -> Option<NaiveDate> {
		match self.kv().get("due") {
			Some(dd) => NaiveDate::parse_from_str(dd, "%Y-%m-%d").ok(),
			None => None,
		}
	}

	/// Classify how urgent this task is.
	pub fn urgency(&self) -> Option<Urgency> {
		let cell = &self._urgency;
		if cell.get().is_none() {
			cell.set(Some(self._build_urgency()));
		}
		cell.get().unwrap()
	}

	fn _build_urgency(&self) -> Option<Urgency> {
		let due = match self.due_date() {
			Some(d) => d,
			None => return None,
		};

		if due < *DATE_TODAY {
			Some(Urgency::Overdue)
		} else if due == *DATE_TODAY {
			Some(Urgency::Today)
		} else if due <= *DATE_SOON {
			Some(Urgency::Soon)
		} else if due <= *DATE_WEEKEND {
			Some(Urgency::ThisWeek)
		} else if due <= *DATE_NEXT_WEEKEND {
			Some(Urgency::NextWeek)
		} else if due <= *DATE_NEXT_MONTH {
			Some(Urgency::NextMonth)
		} else {
			Some(Urgency::Later)
		}
	}

	/// Return the size of this task.
	pub fn tshirt_size(&self) -> Option<TshirtSize> {
		let cell = &self._tshirt_size;
		if cell.get().is_none() {
			cell.set(Some(self._build_tshirt_size()));
		}
		cell.get().unwrap()
	}

	fn _build_tshirt_size(&self) -> Option<TshirtSize> {
		let ctx = self.contexts();

		let mut tmp = ctx.iter().filter(|x| RE_SMALL.is_match(x));
		if tmp.next().is_some() {
			return Some(TshirtSize::Small);
		}

		let mut tmp = ctx.iter().filter(|x| RE_MEDIUM.is_match(x));
		if tmp.next().is_some() {
			return Some(TshirtSize::Medium);
		}

		let mut tmp = ctx.iter().filter(|x| RE_LARGE.is_match(x));
		if tmp.next().is_some() {
			return Some(TshirtSize::Large);
		}

		None
	}

	/// Tags.
	#[allow(dead_code)]
	pub fn tags(&self) -> Vec<String> {
		self._build_tags()
	}

	fn _build_tags(&self) -> Vec<String> {
		let mut tags: Vec<String> = Vec::new();
		for cap in RE_TAG.captures_iter(&self.description) {
			tags.push(cap[1].to_string());
		}
		tags
	}

	/// Contexts.
	pub fn contexts(&self) -> Vec<String> {
		self._build_contexts()
	}

	fn _build_contexts(&self) -> Vec<String> {
		let mut tags: Vec<String> = Vec::new();
		for cap in RE_CONTEXT.captures_iter(&self.description) {
			tags.push(cap[1].to_string());
		}
		tags
	}

	/// Key-Value Tags.
	pub fn kv(&self) -> HashMap<String, String> {
		self._build_kv()
	}

	fn _build_kv(&self) -> HashMap<String, String> {
		let mut kv: HashMap<String, String> = HashMap::new();
		for cap in RE_KV.captures_iter(&self.description) {
			kv.insert(cap[1].to_string(), cap[2].to_string());
		}
		kv
	}

	/// Key used for smart sorting
	pub fn smart_key(&self) -> (Urgency, char, TshirtSize) {
		(
			self.urgency().unwrap_or(Urgency::Soon),
			self.importance().unwrap_or('D'),
			self.tshirt_size().unwrap_or(TshirtSize::Medium),
		)
	}

	/// Write this item to a stream, not in todo.txt format!
	///
	/// Allows for pretty formatting, etc.
	pub fn write_to(
		&self,
		stream: &mut dyn std::io::Write,
		cfg: &ItemFormatConfig,
	) {
		let mut r: String = String::new();

		if self.completion {
			r.push_str("x ");
		} else {
			r.push_str("  ");
		}

		if self.priority == '\0' {
			r.push_str("(?) ");
		} else {
			let style = match self.importance() {
				Some('A') => Style::new().red().bold().force_styling(true),
				Some('B') => Style::new().yellow().bold().force_styling(true),
				Some('C') => Style::new().green().bold().force_styling(true),
				Some(_) => Style::new().bold().force_styling(true),
				_ => Style::new(),
			};
			let paren = format!("({}) ", style.apply_to(self.priority));
			r.push_str(&paren);
		}

		if cfg.with_completion_date {
			if self.completion && self.completion_date.is_some() {
				let date = self
					.completion_date
					.unwrap()
					.format("%Y-%m-%d ")
					.to_string();
				r.push_str(&date);
			} else if self.completion {
				r.push_str("????-??-?? ");
			} else {
				r.push_str("           ");
			}
		}

		if cfg.with_creation_date {
			if self.creation_date.is_some() {
				let date = self
					.creation_date
					.unwrap()
					.format("%Y-%m-%d ")
					.to_string();
				r.push_str(&date);
			} else {
				r.push_str("????-??-?? ");
			}
		}

		let len = cfg.width - console::strip_ansi_codes(&r).len();
		r.push_str(self.description.substring(0, len));

		if self.completion {
			if cfg.colour {
				r = format!(
					"{}",
					Style::new()
						.dim()
						.force_styling(true)
						.apply_to(console::strip_ansi_codes(&r).to_string())
				);
			} else {
				r = console::strip_ansi_codes(&r).to_string();
			}
		} else if !cfg.colour {
			r = console::strip_ansi_codes(&r).to_string();
		}

		if cfg.with_newline {
			writeln!(stream, "{}", r).expect("panik");
		} else {
			write!(stream, "{}", r).expect("panik");
		}
	}
}

/// Config object for the `write_to` method.
impl ItemFormatConfig {
	pub fn new() -> ItemFormatConfig {
		let term = console::Term::stdout();
		let (_height, width) = term.size();
		ItemFormatConfig {
			width: width.into(),
			colour: false,
			with_creation_date: false,
			with_completion_date: false,
			with_newline: true,
		}
	}
}

impl Default for Item {
	fn default() -> Self {
		Self::new()
	}
}

impl Default for ItemFormatConfig {
	fn default() -> Self {
		Self::new()
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use chrono::NaiveDate;

	#[test]
	fn test_debug() {
		let b = Item::new();
		let i = Item {
			completion: false,
			priority: '\0',
			completion_date: None,
			creation_date: None,
			description: "foo bar baz".to_string(),
			..b
		};
		let dbug = format!("{:?}", i);
		assert!(dbug.len() > 1);
	}

	#[test]
	fn test_display() {
		let b = Item::new();
		let i = Item {
			description: "foo bar baz".to_string(),
			..b
		};

		assert_eq!("foo bar baz", format!("{}", i));

		let b = Item::new();
		let i = Item {
			completion: true,
			priority: 'B',
			completion_date: Some(NaiveDate::from_ymd(2010, 1, 1)),
			creation_date: Some(NaiveDate::from_ymd(2000, 12, 31)),
			description: "foo bar baz".to_string(),
			..b
		};

		assert_eq!("x (B) 2010-01-01 2000-12-31 foo bar baz", format!("{}", i));
	}

	#[test]
	fn test_parse() {
		// Parse a complex line
		let i = Item::parse("x (B) 2010-01-01 2000-12-31 foo bar baz");

		assert_eq!(true, i.completion);
		assert_eq!('B', i.priority);
		assert_eq!(NaiveDate::from_ymd(2010, 1, 1), i.completion_date.unwrap());
		assert_eq!(NaiveDate::from_ymd(2000, 12, 31), i.creation_date.unwrap());
		assert_eq!("foo bar baz".to_string(), i.description);
		assert!(i.urgency().is_none());
		assert_eq!('B', i.importance().unwrap());
		assert!(i.tshirt_size().is_none());
		assert_eq!(Vec::<String>::new(), i.tags());
		assert_eq!(Vec::<String>::new(), i.contexts());
		assert_eq!(HashMap::<String, String>::new(), i.kv());

		// Parse a misleading line
		let i = Item::parse("2010-01-01 (A) foo bar baz");

		assert!(!i.completion);
		assert_eq!('\0', i.priority);
		assert!(i.completion_date.is_none());
		assert_eq!(NaiveDate::from_ymd(2010, 1, 1), i.creation_date.unwrap());
		assert_eq!("(A) foo bar baz".to_string(), i.description);
	}

	#[test]
	fn test_kv() {
		let i = Item::parse("(A) foo bar abc:xyz def:123");
		let expected_kv = HashMap::from([
			("abc".to_string(), "xyz".to_string()),
			("def".to_string(), "123".to_string()),
		]);

		assert_eq!('A', i.priority);
		assert_eq!("foo bar abc:xyz def:123".to_string(), i.description);
		assert_eq!(expected_kv, i.kv());
	}

	#[test]
	fn test_due_date() {
		let i = Item::parse("(A) foo bar due:1980-06-01");

		assert_eq!(NaiveDate::from_ymd(1980, 6, 1), i.due_date().unwrap());
	}

	#[test]
	fn test_urgency() {
		let i = Item::parse("(A) foo bar due:1970-06-01");
		assert_eq!(Urgency::Overdue, i.urgency().unwrap());

		let i = Item::parse(&format!(
			"(A) foo bar due:{}",
			Utc::now().date_naive().format("%Y-%m-%d")
		));
		assert_eq!(Urgency::Today, i.urgency().unwrap());

		let i = Item::parse(&format!(
			"(A) foo bar due:{}",
			(Utc::now().date_naive() + Duration::days(1)).format("%Y-%m-%d")
		));
		assert_eq!(Urgency::Soon, i.urgency().unwrap());

		let i = Item::parse(&format!(
			"(A) foo bar due:{}",
			(Utc::now().date_naive() + Duration::days(18)).format("%Y-%m-%d")
		));
		assert_eq!(Urgency::NextMonth, i.urgency().unwrap());

		let i = Item::parse("(A) foo bar due:3970-06-01");
		assert_eq!(Urgency::Later, i.urgency().unwrap());
	}

	#[test]
	fn test_tags() {
		let i = Item::parse("(A) +Foo +foo bar+baz +bam");
		let expected_tags = Vec::from([
			"Foo".to_string(),
			"foo".to_string(),
			"bam".to_string(),
		]);
		assert_eq!(expected_tags, i.tags());
	}

	#[test]
	fn test_contexts() {
		let i = Item::parse("(A) @Foo @foo bar@baz @bam");
		let expected_ctx = Vec::from([
			"Foo".to_string(),
			"foo".to_string(),
			"bam".to_string(),
		]);
		assert_eq!(expected_ctx, i.contexts());
	}

	#[test]
	fn test_tshirt_size() {
		let i = Item::parse("@M Barble");
		assert_eq!(TshirtSize::Medium, i.tshirt_size().unwrap());

		let i = Item::parse("(A) Fooble @XxL Barble");
		assert_eq!(TshirtSize::Large, i.tshirt_size().unwrap());

		let i = Item::parse("Barble");
		assert!(i.tshirt_size().is_none());
	}
}

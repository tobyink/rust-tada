use chrono::{Datelike, Duration, NaiveDate, Utc, Weekday};
use console::Style;
use freezebox::FreezeBox;
use lazy_static::lazy_static;
use regex::Regex;
use std::collections::HashMap;
use std::fmt;
use substring::Substring;

pub struct ItemFormatConfig {
	pub width: usize,
	pub colour: bool,
	pub with_creation_date: bool,
	pub with_completion_date: bool,
	pub with_line_numbers: bool,
	pub with_newline: bool,
	pub line_number_digits: usize,
}

/// An item in a todo list.
pub struct Item {
	line_number: usize,
	completion: bool,
	priority: char,
	completion_date: Option<NaiveDate>,
	creation_date: Option<NaiveDate>,
	description: String,
	_importance: FreezeBox<Option<char>>,
	_due_date: FreezeBox<Option<NaiveDate>>,
	_urgency: FreezeBox<Option<Urgency>>,
	_tshirt_size: FreezeBox<Option<TshirtSize>>,
	_tags: FreezeBox<Vec<String>>,
	_contexts: FreezeBox<Vec<String>>,
	_kv: FreezeBox<HashMap<String, String>>,
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

impl Clone for Item {
	fn clone(&self) -> Self {
		Item {
			line_number: self.line_number,
			completion: self.completion,
			priority: self.priority,
			completion_date: self.completion_date,
			creation_date: self.creation_date,
			description: self.description.clone(),
			..Item::new()
		}
	}
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
		if self.completion {
			write!(f, "x ")?;
		}

		if self.priority != '\0' {
			write!(f, "({}) ", self.priority)?;
		}

		if self.completion {
			if let Some(d) = self.completion_date {
				write!(f, "{} ", d.format("%Y-%m-%d"))?;
			}
		}

		if let Some(d) = self.creation_date {
			write!(f, "{} ", d.format("%Y-%m-%d"))?;
		}

		write!(f, "{}", self.description)
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
			line_number: 0,
			completion: false,
			priority: '\0',
			completion_date: None,
			creation_date: None,
			description: String::new(),
			_importance: FreezeBox::default(),
			_due_date: FreezeBox::default(),
			_urgency: FreezeBox::default(),
			_tshirt_size: FreezeBox::default(),
			_tags: FreezeBox::default(),
			_contexts: FreezeBox::default(),
			_kv: FreezeBox::default(),
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

	/// Whether the task is complete.
	pub fn completion(&self) -> bool {
		self.completion
	}

	/// Set indicator of whether the task is complete.
	pub fn set_completion(&mut self, x: bool) {
		self.completion = x;
	}

	/// Line number indicator (sometimes zero).
	pub fn line_number(&self) -> usize {
		self.line_number
	}

	/// Set line number indicator for the task.
	pub fn set_line_number(&mut self, x: usize) {
		self.line_number = x;
	}

	/// Task priority/importance as given in a todo.txt file.
	///
	/// A is highest, then B and C. D should be considered normal. E is low priority.
	/// Any uppercase letter is valid, but letters after E are not especially meaningful.
	///
	/// The importance() method is better.
	pub fn priority(&self) -> char {
		self.priority
	}

	/// Set task priority.
	pub fn set_priority(&mut self, x: char) {
		self.priority = x;
	}

	/// Completion date.
	///
	/// Often none.
	pub fn completion_date(&self) -> Option<NaiveDate> {
		self.completion_date
	}

	/// Set the completion date to a given date.
	pub fn set_completion_date(&mut self, x: NaiveDate) {
		self.completion_date = Some(x);
	}

	/// Set the completion date to None.
	pub fn clear_completion_date(&mut self) {
		self.completion_date = None;
	}

	/// Task creation date.
	///
	/// Often none.
	pub fn creation_date(&self) -> Option<NaiveDate> {
		self.creation_date
	}

	/// Set the task creation date to a given date.
	pub fn set_creation_date(&mut self, x: NaiveDate) {
		self.creation_date = Some(x);
	}

	/// Set the task creation date to None.
	pub fn clear_creation_date(&mut self) {
		self.creation_date = None;
	}

	/// Task description.
	pub fn description(&self) -> String {
		self.description.clone()
	}

	/// Set the task description.
	///
	/// Internally clears cached tags, etc.
	pub fn set_description(&mut self, x: String) {
		self._importance = FreezeBox::default();
		self._due_date = FreezeBox::default();
		self._urgency = FreezeBox::default();
		self._tshirt_size = FreezeBox::default();
		self._tags = FreezeBox::default();
		self._contexts = FreezeBox::default();
		self._kv = FreezeBox::default();
		self.description = x;
	}

	/// Return the importance of this task.
	///
	/// Basically the same as priority, except all letters after E
	/// are treated as being the same as E. Returns None for \0.
	pub fn importance(&self) -> Option<char> {
		if !self._importance.is_initialized() {
			self._importance
				.lazy_init(self._build_importance());
		}
		*self._importance
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
		if !self._due_date.is_initialized() {
			self._due_date.lazy_init(self._build_due_date());
		}
		*self._due_date
	}

	fn _build_due_date(&self) -> Option<NaiveDate> {
		match self.kv().get("due") {
			Some(dd) => NaiveDate::parse_from_str(dd, "%Y-%m-%d").ok(),
			None => None,
		}
	}

	/// Classify how urgent this task is.
	pub fn urgency(&self) -> Option<Urgency> {
		if !self._urgency.is_initialized() {
			self._urgency.lazy_init(self._build_urgency());
		}
		*self._urgency
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
		if !self._tshirt_size.is_initialized() {
			self._tshirt_size
				.lazy_init(self._build_tshirt_size());
		}
		*self._tshirt_size
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
		if !self._tags.is_initialized() {
			self._tags.lazy_init(self._build_tags());
		}
		// Need to return a copy
		(*self._tags).to_vec()
	}

	fn _build_tags(&self) -> Vec<String> {
		let mut tags: Vec<String> = Vec::new();
		for cap in RE_TAG.captures_iter(&self.description) {
			tags.push(cap[1].to_string());
		}
		tags
	}

	/// Boolean indicating whether a task has a particular tag.
	pub fn has_tag(&self, tag: &str) -> bool {
		let real_tag = match tag.chars().next() {
			Some('+') => tag.get(1..).unwrap(),
			_ => tag,
		};
		let real_tag = real_tag.to_lowercase();
		self.tags()
			.iter()
			.any(|t| t.to_lowercase().as_str() == real_tag)
	}

	/// Contexts.
	pub fn contexts(&self) -> Vec<String> {
		if !self._contexts.is_initialized() {
			self._contexts.lazy_init(self._build_contexts());
		}
		// Need to return a copy
		(*self._contexts).to_vec()
	}

	fn _build_contexts(&self) -> Vec<String> {
		let mut tags: Vec<String> = Vec::new();
		for cap in RE_CONTEXT.captures_iter(&self.description) {
			tags.push(cap[1].to_string());
		}
		tags
	}

	/// Boolean indicating whether a task has a particular context.
	pub fn has_context(&self, ctx: &str) -> bool {
		let real_ctx = match ctx.chars().next() {
			Some('@') => ctx.get(1..).unwrap(),
			_ => ctx,
		};
		let real_ctx = real_ctx.to_lowercase();
		self.contexts()
			.iter()
			.any(|c| c.to_lowercase().as_str() == real_ctx)
	}

	/// Key-Value Tags.
	pub fn kv(&self) -> HashMap<String, String> {
		if !self._kv.is_initialized() {
			self._kv.lazy_init(self._build_kv());
		}
		// Need to return a copy
		let mut kv_clone: HashMap<String, String> = HashMap::new();
		for (k, v) in &*self._kv {
			kv_clone.insert(k.clone(), v.clone());
		}
		kv_clone
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

		if cfg.with_line_numbers {
			r.push_str(
				format!(
					"#{:0width$} ",
					self.line_number(),
					width = cfg.line_number_digits
				)
				.as_str(),
			)
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
	/// Constructor for item format config, given an output width
	pub fn new(width: usize) -> ItemFormatConfig {
		ItemFormatConfig {
			width,
			colour: false,
			with_creation_date: false,
			with_completion_date: false,
			with_line_numbers: false,
			with_newline: true,
			line_number_digits: 2,
		}
	}

	/// Alternative constructor, which detects width from the terminal
	pub fn new_based_on_terminal() -> ItemFormatConfig {
		let term = console::Term::stdout();
		let (_height, width) = term.size();
		Self::new(width.into())
	}
}

impl Default for Item {
	fn default() -> Self {
		Self::new()
	}
}

impl Default for ItemFormatConfig {
	fn default() -> Self {
		Self::new_based_on_terminal()
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
		assert!(i.has_tag("Foo"));
		assert!(i.has_tag("fOO"));
		assert!(!i.has_tag("Fool"));
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
		assert!(i.has_context("Foo"));
		assert!(i.has_context("fOO"));
		assert!(!i.has_context("Fool"));
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

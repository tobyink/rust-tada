//! Types related to individual tasks.
//!
//! # Examples
//!
//! ```
//! use chrono::NaiveDate;
//! use tada::item::{Importance, Item, TshirtSize, Urgency};
//!
//! let mut i = Item::parse("(A) clean my @home @L");
//!
//! assert_eq!(Some(Importance::A), i.importance());
//! assert_eq!("clean my @home @L", i.description());
//! assert!(i.has_context("home"));
//! assert!(i.has_context("l"));
//! assert_eq!(Some(TshirtSize::Large), i.tshirt_size());
//!
//! i.set_completion(true);
//! i.set_completion_date(NaiveDate::from_ymd_opt(2022, 12, 1).unwrap());
//! println!("{}", i);
//! ```

use chrono::{Datelike, Duration, NaiveDate, Utc, Weekday};
use date_time_parser::DateParser as NaturalDateParser;
use freezebox::FreezeBox;
use lazy_static::lazy_static;
use regex::Regex;
use std::collections::HashMap;
use std::fmt;

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
		.pred_opt()
		.unwrap()
	};
}

/// Five levels of importance are defined.
#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord, Hash)]
pub enum Importance {
	/// Critical
	A,
	/// Important
	B,
	/// Semi-important
	C,
	/// Normal
	D,
	/// Unimportant
	E,
}

impl Importance {
	/// Get an importance from a letter.
	pub fn from_char(c: char) -> Option<Self> {
		match c {
			'A' => Some(Self::A),
			'B' => Some(Self::B),
			'C' => Some(Self::C),
			'D' => Some(Self::D),
			'E'..='Z' => Some(Self::E),
			_ => None,
		}
	}

	/// Returns a letter representing this importance.
	pub fn to_char(&self) -> char {
		match self {
			Self::A => 'A',
			Self::B => 'B',
			Self::C => 'C',
			Self::D => 'D',
			Self::E => 'E',
		}
	}

	/// Returns a heading suitable for items of this importance.
	pub fn to_string(&self) -> &str {
		match self {
			Self::A => "Critical",
			Self::B => "Important",
			Self::C => "Semi-important",
			Self::D => "Normal",
			Self::E => "Unimportant",
		}
	}

	/// Returns a list of known importances, in a sane order.
	pub fn all() -> Vec<Self> {
		Vec::from([Self::A, Self::B, Self::C, Self::D, Self::E])
	}
}

impl Default for Importance {
	/// Default is D.
	fn default() -> Self {
		Self::D
	}
}

/// Seven levels of urgency are defined.
#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord, Hash)]
pub enum Urgency {
	/// A due date earlier than today.
	Overdue,
	/// A due date today.
	Today,
	/// A due date tomorrow or overmorrow.
	Soon,
	/// A due date by the end of this week. Note that if it is Friday or later, any
	/// tasks due this week will fall into the `Today` or `Soon` urgencies instead.
	ThisWeek,
	/// A due date by the end of next week.
	NextWeek,
	/// A due date by the end of next month.
	///
	/// There is no `ThisMonth` urgency because for almost half the month any tasks
	/// would fall into the `ThisWeek` or `NextWeek` urgencies instead, making
	/// `ThisMonth` fairly useless.
	NextMonth,
	/// Any due date after the end of next month.
	Later,
}

impl Urgency {
	/// Calculate urgency from a due date.
	pub fn from_due_date(due: NaiveDate) -> Self {
		if due < *DATE_TODAY {
			Self::Overdue
		} else if due == *DATE_TODAY {
			Self::Today
		} else if due <= *DATE_SOON {
			Self::Soon
		} else if due <= *DATE_WEEKEND {
			Self::ThisWeek
		} else if due <= *DATE_NEXT_WEEKEND {
			Self::NextWeek
		} else if due <= *DATE_NEXT_MONTH {
			Self::NextMonth
		} else {
			Self::Later
		}
	}

	/// Returns a heading suitable for items of this urgency.
	pub fn to_string(&self) -> &str {
		match self {
			Self::Overdue => "Overdue",
			Self::Today => "Today",
			Self::Soon => "Soon",
			Self::ThisWeek => "This week",
			Self::NextWeek => "Next week",
			Self::NextMonth => "Next month",
			Self::Later => "Later",
		}
	}

	/// Returns a list of known urgencies, in a sane order.
	pub fn all() -> Vec<Self> {
		Vec::from([
			Self::Overdue,
			Self::Today,
			Self::Soon,
			Self::ThisWeek,
			Self::NextWeek,
			Self::NextMonth,
			Self::Later,
		])
	}
}

impl Default for Urgency {
	/// Default is soon, but you should rely on the default as little as possible.
	/// It is useful when sorting tasks by urgency.
	fn default() -> Self {
		Self::Soon
	}
}

/// Three sizes are defined.
#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord, Hash)]
pub enum TshirtSize {
	Small,
	Medium,
	Large,
}

impl TshirtSize {
	/// Returns a heading suitable for items of this size.
	pub fn to_string(&self) -> &str {
		match self {
			Self::Small => "Small",
			Self::Medium => "Medium",
			Self::Large => "Large",
		}
	}

	/// Returns a list of known sizes, in a sane order.
	pub fn all() -> Vec<Self> {
		Vec::from([Self::Small, Self::Medium, Self::Large])
	}
}

impl Default for TshirtSize {
	/// Default is medium.
	fn default() -> Self {
		Self::Medium
	}
}

/// An item in a todo list.
///
/// # Examples
///
/// ```
/// use tada::{Importance, Item};
/// let i = Item::parse("(A) clean my @home");
/// assert_eq!(Some(Importance::A), i.importance());
/// assert_eq!("clean my @home", i.description());
/// assert!(i.has_context("home"));
/// ```
pub struct Item {
	line_number: usize,
	completion: bool,
	priority: char,
	completion_date: Option<NaiveDate>,
	creation_date: Option<NaiveDate>,
	description: String,
	_importance: FreezeBox<Option<Importance>>,
	_due_date: FreezeBox<Option<NaiveDate>>,
	_start_date: FreezeBox<Option<NaiveDate>>,
	_urgency: FreezeBox<Option<Urgency>>,
	_tshirt_size: FreezeBox<Option<TshirtSize>>,
	_tags: FreezeBox<Vec<String>>,
	_contexts: FreezeBox<Vec<String>>,
	_kv: FreezeBox<HashMap<String, String>>,
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
			_start_date: FreezeBox::default(),
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

	/// Create a version of this item but representing a completed task.
	pub fn but_done(&self, include_date: bool) -> Item {
		let mut i = self.clone();
		i.set_completion(true);
		if include_date {
			i.set_completion_date(*DATE_TODAY);
			if i.creation_date().is_none() {
				i.set_creation_date(*DATE_TODAY);
			}
		}
		i
	}

	/// Provide zen-like calm by rescheduling an overdue task.
	pub fn zen(&self) -> Item {
		if self.urgency() == Some(Urgency::Overdue) {
			let mut new = self.clone();
			let important = matches!(
				new.importance(),
				Some(Importance::A) | Some(Importance::B)
			);
			let small = matches!(new.tshirt_size(), Some(TshirtSize::Small));
			let new_urgency = if important && small {
				Urgency::Soon
			} else if important || small {
				Urgency::NextWeek
			} else {
				Urgency::NextMonth
			};
			new.set_urgency(new_urgency);
			return new;
		}
		self.clone()
	}

	/// Pull a task forward to being done with a new urgency, also clearing any start date.
	pub fn but_pull(&self, new_urgency: Urgency) -> Item {
		let mut new = self.clone();
		new.set_urgency(new_urgency);

		let re = Regex::new(r"start:(?:[^\s:]+)").unwrap();
		let new_start = format!("start:{}", DATE_TODAY.format("%Y-%m-%d"));
		new.set_description(format!(
			"{}",
			re.replace(&new.description, new_start)
		));

		new
	}

	/// Performs a bunch of small fixes on the item syntax.
	pub fn fixup(&self, warnings: bool) -> Item {
		let maybe_warn = |w| {
			if warnings {
				eprintln!("{}", w);
			}
		};
		let mut new = self.clone();

		if new.priority() == '\0' {
			maybe_warn(String::from("Hint: a task can be given an importance be prefixing it with a parenthesized capital letter, like `(A)`."));
		}

		for slot in ["due", "start"] {
			match new.kv().get(slot) {
				Some(given_date) => {
					if NaiveDate::parse_from_str(given_date, "%Y-%m-%d")
						.is_err()
					{
						let processed_date = given_date.replace('_', " ");
						if let Some(naive_date) =
							NaturalDateParser::parse(&processed_date)
						{
							new.set_description(new.description().replace(
								&format!("{}:{}", slot, given_date),
								&format!(
									"{}:{}",
									slot,
									naive_date.format("%Y-%m-%d")
								),
							));
							maybe_warn(format!(
								"Notice: {} date `{}` changed to `{}`.",
								slot,
								given_date,
								naive_date.format("%Y-%m-%d")
							));
						} else {
							maybe_warn(format!("Notice: {} date `{}` should be in YYYY-MM-DD format.", slot, given_date));
						}
					}
				}
				None => {
					if slot == "due" {
						maybe_warn(format!("Hint: a task can be given a {} date by including `{}:YYYY-MM-DD`.", slot, slot));
					}
				}
			}
		}

		if new.tshirt_size().is_none() {
			maybe_warn(String::from("Hint: a task can be given a size by including `@S`, `@M`, or `@L`."));
		}

		if new.description().len() > 120 {
			maybe_warn(String::from("Hint: long descriptions can make a task list slower to skim read."));
		} else if new.description().len() < 30 {
			maybe_warn(String::from("Hint: short descriptions can make it hard to remember what a task means!"));
		}

		new
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
	/// Basically the same as priority, except it's an enum and all letters after E
	/// are treated as being the same as E.
	pub fn importance(&self) -> Option<Importance> {
		if !self._importance.is_initialized() {
			self._importance
				.lazy_init(self._build_importance());
		}
		*self._importance
	}

	fn _build_importance(&self) -> Option<Importance> {
		Importance::from_char(self.priority)
	}

	/// Set the item's importance.
	pub fn set_importance(&mut self, i: Importance) {
		self.priority = i.to_char();
		self._importance = FreezeBox::default();
	}

	/// Set the item's importance.
	pub fn clear_importance(&mut self) {
		self.priority = '\0';
		self._importance = FreezeBox::default();
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

	/// Return the date when this task may be started.
	pub fn start_date(&self) -> Option<NaiveDate> {
		if !self._start_date.is_initialized() {
			self._start_date
				.lazy_init(self._build_start_date());
		}
		*self._start_date
	}

	fn _build_start_date(&self) -> Option<NaiveDate> {
		match self.kv().get("start") {
			Some(dd) => NaiveDate::parse_from_str(dd, "%Y-%m-%d").ok(),
			None => None,
		}
	}

	/// A task is startable if it doesn't have a start date which is in the future.
	pub fn is_startable(&self) -> bool {
		match self.start_date() {
			Some(day) => day <= *DATE_TODAY,
			None => true,
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
		self.due_date().map(Urgency::from_due_date)
	}

	/// Set task urgency.
	pub fn set_urgency(&mut self, urg: Urgency) {
		let mut d = match urg {
			Urgency::Overdue => DATE_TODAY.pred_opt().unwrap(),
			Urgency::Today => *DATE_TODAY,
			Urgency::Soon => *DATE_SOON,
			Urgency::ThisWeek => *DATE_WEEKEND,
			Urgency::NextWeek => *DATE_NEXT_WEEKEND,
			Urgency::NextMonth => *DATE_NEXT_MONTH,
			Urgency::Later => *DATE_TODAY + Duration::days(183), // about 6 months
		};
		// Work and school tasks should be rescheduled from Saturday/Sunday.
		if urg > Urgency::Today
			&& (self.has_context("work") || self.has_context("school"))
		{
			d = match format!("{}", d.format("%u")).as_str() {
				"6" => d.pred_opt().unwrap(),
				"7" => d.pred_opt().unwrap().pred_opt().unwrap(),
				_ => d,
			};
		}

		let formatted = d.format("%Y-%m-%d");

		match self.kv().get("due") {
			Some(str) => {
				self.set_description(self.description().replace(
					&format!("due:{str}"),
					&format!("due:{formatted}"),
				))
			}
			None => self.set_description(format!(
				"{} due:{formatted}",
				self.description()
			)),
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
	pub fn smart_key(&self) -> (Urgency, Importance, TshirtSize) {
		(
			self.urgency().unwrap_or_default(),
			self.importance().unwrap_or_default(),
			self.tshirt_size().unwrap_or_default(),
		)
	}
}

impl Default for Item {
	fn default() -> Self {
		Self::new()
	}
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
			completion_date: Some(NaiveDate::from_ymd_opt(2010, 1, 1).unwrap()),
			creation_date: Some(NaiveDate::from_ymd_opt(2000, 12, 31).unwrap()),
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
		assert_eq!(
			NaiveDate::from_ymd_opt(2010, 1, 1).unwrap(),
			i.completion_date.unwrap()
		);
		assert_eq!(
			NaiveDate::from_ymd_opt(2000, 12, 31).unwrap(),
			i.creation_date.unwrap()
		);
		assert_eq!("foo bar baz".to_string(), i.description);
		assert!(i.urgency().is_none());
		assert_eq!(Some(Importance::B), i.importance());
		assert_eq!(None, i.tshirt_size());
		assert_eq!(Vec::<String>::new(), i.tags());
		assert_eq!(Vec::<String>::new(), i.contexts());
		assert_eq!(HashMap::<String, String>::new(), i.kv());

		// Parse a misleading line
		let i = Item::parse("2010-01-01 (A) foo bar baz");

		assert!(!i.completion);
		assert_eq!('\0', i.priority);
		assert!(i.completion_date.is_none());
		assert_eq!(
			NaiveDate::from_ymd_opt(2010, 1, 1).unwrap(),
			i.creation_date.unwrap()
		);
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

		assert_eq!(
			NaiveDate::from_ymd_opt(1980, 6, 1).unwrap(),
			i.due_date().unwrap()
		);
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

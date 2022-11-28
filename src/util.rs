//! Useful functions for manipulating todo lists.
//!
//! These mostly operate on `Vec<&Item>`.

use crate::action::SortOrder;
use crate::item::{Item, TshirtSize, Urgency};
use std::collections::HashMap;

/// Sort Vec<&Item> in a variety of ways.
pub fn sort_items_by<'a>(
	sortby: &'a str,
	items: Vec<&'a Item>,
) -> Vec<&'a Item> {
	match SortOrder::from_string(sortby) {
		Ok(o) => o.sort_items(items),
		Err(_) => {
			eprintln!("Invalid sort order");
			items
		}
	}
}

/// Filter Vec<&Item> by an @context.
pub fn find_items_by_context<'a>(
	term: &'a str,
	items: Vec<&'a Item>,
) -> Vec<&'a Item> {
	items
		.into_iter()
		.filter(|i| i.has_context(term))
		.collect()
}

/// Filter Vec<&Item> by a +tag.
pub fn find_items_by_tag<'a>(
	term: &'a str,
	items: Vec<&'a Item>,
) -> Vec<&'a Item> {
	items
		.into_iter()
		.filter(|i| i.has_tag(term))
		.collect()
}

/// Filter Vec<&Item> by a #linenumber.
pub fn find_items_by_line_number<'a>(
	term: &'a str,
	items: Vec<&'a Item>,
) -> Vec<&'a Item> {
	let n: usize = term.get(1..).unwrap().parse().unwrap();
	items
		.into_iter()
		.filter(|i| i.line_number() == n)
		.collect()
}

/// Filter Vec<&Item> by a string match.
pub fn find_items_by_string<'a>(
	term: &'a str,
	items: Vec<&'a Item>,
) -> Vec<&'a Item> {
	let lc_term = term.to_lowercase();
	items
		.into_iter()
		.filter(|i| {
			i.description()
				.to_lowercase()
				.contains(&lc_term)
		})
		.collect()
}

/// Group a Vec<&Item> into categories based on task urgency.
pub fn group_items_by_urgency(
	items: Vec<&Item>,
) -> HashMap<Urgency, Vec<&Item>> {
	let mut out: HashMap<Urgency, Vec<&Item>> = HashMap::new();
	for i in items {
		let list = out
			.entry(i.urgency().unwrap_or(Urgency::Soon))
			.or_insert_with(Vec::new);
		list.push(i);
	}
	out
}

/// Group a Vec<&Item> into categories based on task size.
pub fn group_items_by_size(
	items: Vec<&Item>,
) -> HashMap<TshirtSize, Vec<&Item>> {
	let mut out: HashMap<TshirtSize, Vec<&Item>> = HashMap::new();
	for i in items {
		let list = out
			.entry(i.tshirt_size().unwrap_or(TshirtSize::Medium))
			.or_insert_with(Vec::new);
		list.push(i);
	}
	out
}

/// Group a Vec<&Item> into categories based on task improtance.
pub fn group_items_by_importance(
	items: Vec<&Item>,
) -> HashMap<char, Vec<&Item>> {
	let mut out: HashMap<char, Vec<&Item>> = HashMap::new();
	for i in items {
		let list = out
			.entry(i.importance().unwrap_or('D'))
			.or_insert_with(Vec::new);
		list.push(i);
	}
	out
}

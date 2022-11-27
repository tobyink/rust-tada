use crate::action::*;
use crate::list::Line;
use clap::{ArgMatches, Command};
use rand::seq::SliceRandom;

/// Options for the `zen` subcommand.
pub fn get_action() -> Action {
	let name = String::from("zen");
	let mut command = Command::new("zen").about("Automatically reschedule overdue tasks")
		.after_help(
			"Zen will reschedule any overdue tasks on your todo list. It does not consult you\n\
			to ask for a new due date, but guesses when a sensible due date might be."
		);
	command = FileType::TodoTxt.add_args(command);
	command = Outputter::add_args_minimal(command);
	Action { name, command }
}

/// Execute the `zen` subcommand.
pub fn execute(args: &ArgMatches) {
	let mut outputter = Outputter::from_argmatches_minimal(args);
	let todo_filename = FileType::TodoTxt.filename(args);
	let list = List::from_url(todo_filename.clone())
		.expect("Could not read todo list");
	let mut new_list = List::new();

	for line in list.lines {
		match line.kind {
			LineKind::Item => {
				let new_line = Line::from_item(line.item.unwrap().zen());
				new_list.lines.push(new_line);
			}
			_ => new_list.lines.push(line),
		}
	}

	new_list.to_url(todo_filename);

	outputter.write_status(String::from(zen_quote()));
}

pub fn zen_quote() -> &'static str {
	let quotes = Vec::from([
		// Marcus Aurelius
		"The nearer a man comes to a calm mind, the closer he is to strength.",
		"It is in your power to withdraw yourself whenever you desire. Perfect tranquility\n\
		within consists in the good ordering of the mind, the realm of your own.",
		// Bhagavad Gita
		"All sorrows are destroyed upon attainment of tranquility. The intellect of such\n\
		a tranquil person soon becomes completely steady.",
		// Chris Bradford
		"A samurai must remain calm at all times even in the face of danger.",
		// Buddhism
		"Those who are free of resentful thoughts surely find peace.",
		"Passaddhi, calm or tranquillity, is the fifth factor of enlightenment.",
		// Pema Chodron
		"You are the sky. Everything else... it's just the weather.",
		// Cicero
		"The pursuit, even of the best things, ought to be calm and tranquil.",
		"We think a happy life consists in tranquility of mind.",
		// Khaled Hosseini
		"Quiet is peace. Tranquility. Quiet is turning down the volume knob on life. Silence\n\
		is pushing the off button. Shutting it down. All of it.",
		// Mehmet Murat ildan
		"Tranquillity is a fertile soil where you can plant and reap the solutions!",
		// Nelson Mandela
		"I never lose; either win or learn.",
		// Alfred Tennyson
		"The noonday quiet holds the hill.",
		// Zen
		"No snowflake ever falls in the wrong place.",
		"When you reach the top of the mountain, keep climbing.",
	]);

	quotes.choose(&mut rand::thread_rng()).unwrap()
}

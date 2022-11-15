//use tada::*;
use clap::Command;
use tada::Action;

/// Get a list of valid subcommands.
fn actions() -> Vec<Action> {
	Vec::from([
		tada::action::add::get_action(),
		tada::action::show::get_action(),
		tada::action::important::get_action(),
		tada::action::urgent::get_action(),
		tada::action::quick::get_action(),
	])
}

/// Main body of the `tada` command.
fn main() {
	let mut cmd = Command::new("tada")
		.version("0.1.0")
		.about("A todo list manager")
		.subcommand_required(true)
		.term_width(80)
		.allow_external_subcommands(true);

	for action in actions() {
		cmd = cmd.subcommand(action.command);
	}

	let matches = cmd.get_matches();
	match matches.subcommand() {
		Some(("add", args)) => tada::action::add::execute(args),
		Some(("show", args)) => tada::action::show::execute(args),
		Some(("important", args)) => tada::action::important::execute(args),
		Some(("urgent", args)) => tada::action::urgent::execute(args),
		Some(("quick", args)) => tada::action::quick::execute(args),
		_ => panic!("hmmm"),
	}
}

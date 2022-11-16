use clap::Command;
use std::process;
use tada::Action;

/// Get a list of valid subcommands.
fn actions() -> Vec<Action> {
	Vec::from([
		tada::action::add::get_action(),
		tada::action::edit::get_action(),
		tada::action::find::get_action(),
		tada::action::show::get_action(),
		tada::action::important::get_action(),
		tada::action::urgent::get_action(),
		tada::action::quick::get_action(),
		tada::action::archive::get_action(),
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

	let matches = cmd.clone().get_matches();
	match matches.subcommand() {
		Some(("add", args)) => tada::action::add::execute(args),
		Some(("show", args)) => tada::action::show::execute(args),
		Some(("important", args)) => tada::action::important::execute(args),
		Some(("urgent", args)) => tada::action::urgent::execute(args),
		Some(("quick", args)) => tada::action::quick::execute(args),
		Some(("find", args)) => tada::action::find::execute(args),
		Some(("archive", args)) => tada::action::archive::execute(args),
		Some(("edit", args)) => tada::action::edit::execute(args),
		Some((tag, _)) => match tag.chars().next() {
			Some('@') | Some('+') => tada::action::find::execute_shortcut(tag),
			_ => {
				cmd.print_help().unwrap();
				process::exit(1);
			}
		},
		None => {
			cmd.print_help().unwrap();
			process::exit(1);
		}
	}
}

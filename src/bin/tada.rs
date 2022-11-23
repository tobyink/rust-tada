use clap::Command;
use std::process;
use tada::action;
use tada::action::Action;

/// Get a list of valid subcommands.
fn actions() -> Vec<Action> {
	Vec::from([
		action::add::get_action(),
		action::remove::get_action(),
		action::edit::get_action(),
		action::pull::get_action(),
		action::done::get_action(),
		action::find::get_action(),
		action::show::get_action(),
		action::important::get_action(),
		action::urgent::get_action(),
		action::quick::get_action(),
		action::archive::get_action(),
		action::tidy::get_action(),
		action::zen::get_action(),
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
	let subcommand = match matches.subcommand() {
		Some(s) => s,
		None => {
			cmd.print_help().unwrap();
			process::exit(1);
		}
	};

	match subcommand {
		("add", args) => action::add::execute(args),
		("archive", args) => action::archive::execute(args),
		("done", args) => action::done::execute(args),
		("edit", args) => action::edit::execute(args),
		("find", args) => action::find::execute(args),
		("important", args) => action::important::execute(args),
		("pull", args) => action::pull::execute(args),
		("quick", args) => action::quick::execute(args),
		("remove", args) => action::remove::execute(args),
		("show", args) => action::show::execute(args),
		("tidy", args) => action::tidy::execute(args),
		("urgent", args) => action::urgent::execute(args),
		("zen", args) => action::zen::execute(args),
		(tag, _) => match tag.chars().next() {
			Some('@') | Some('+') | Some('#') => {
				action::find::execute_shortcut(tag)
			}
			_ => {
				cmd.print_help().unwrap();
				process::exit(1);
			}
		},
	}
}

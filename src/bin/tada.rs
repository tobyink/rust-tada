//use tada::*;
use clap::Command;
use tada::Action;

fn cli() -> Command {
	let mut cmd = Command::new("tada")
		.about("A todo list manager")
		.subcommand_required(true)
		.allow_external_subcommands(true);
	for action in actions().iter() {
		cmd = cmd.subcommand(action.command.clone());
	}
	cmd
}

fn actions() -> Vec<Action> {
	Vec::from([tada::action::show::get_action()])
}

fn main() {
	let matches = cli().get_matches();
	match matches.subcommand() {
		Some(("show", args)) => tada::action::show::execute(args),
		_ => panic!("hmmm"),
	}
}

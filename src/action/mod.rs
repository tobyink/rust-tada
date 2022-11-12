use clap::{Arg, Command};

pub struct Action {
	pub name: String,
	pub command: Command,
}

impl Action {
	fn _add_file_options(cmd: Command) -> Command {
		cmd.arg(
			Arg::new("file")
				.short('f')
				.long("file")
				.value_name("FILE")
				.help("the path to todo.txt"),
		)
	}
}

pub mod show;

use clap::{Arg, ArgAction, Command as App};

/* ASCII format of the project logo */
const ASCII_LOGO: &str = "
 ``    ````````````    ````   ```````````    ```````````
:NNs `hNNNNNNNNNNNNh` sNNNy   yNNNNNNNNNN+   dNNNNNNNNNN:
/MMMydMMyyyyyyydMMMMdhMMMMy   yMMMyyyhMMMo   dMMMyyydMMM/
/MMMMMMM`      oMMMMMMMMMMy   yMMM`  -MMMo   dMMN   /MMM/
/MMMs:::hhhs   oMMM+:::MMMNhhhNMMMdhhdMMMmhhhNMMN   /MMM/
:mmm/   dmmh   +mmm-  `mmmmmmmmmmmmmmmmmmmmmmmmmd   /mmm:
 ```    ```     ```    ``````````````````````````    ```";

/**
 * Parse command line arguments using clap.
 *
 * @return App
 */
pub fn get_args() -> App {
	App::new(env!("CARGO_PKG_NAME"))
		.version(env!("CARGO_PKG_VERSION"))
		.author(env!("CARGO_PKG_AUTHORS"))
		.about(format!(
			"{} {}\n{}\n{}\n\n{}",
			env!("CARGO_PKG_NAME"),
			env!("CARGO_PKG_VERSION"),
			env!("CARGO_PKG_AUTHORS"),
			env!("CARGO_PKG_DESCRIPTION"),
			"Press '?' while running the terminal UI to see key bindings."
		))
		.before_help(ASCII_LOGO)
		.arg(
			Arg::new("accent-color")
				.short('a')
				.long("accent-color")
				.value_name("COLOR")
				.default_value("white")
				.help("Set the accent color using hex or color name")
				.num_args(1),
		)
		.arg(
			Arg::new("color")
				.short('c')
				.long("color")
				.value_name("COLOR")
				.default_value("darkgray")
				.help("Set the main color using hex or color name")
				.num_args(1),
		)
		.arg(
			Arg::new("rate")
				.short('t')
				.long("tickrate")
				.value_name("MS")
				.default_value("250")
				.help("Set the refresh rate of the terminal")
				.num_args(1),
		)
		.arg(
			Arg::new("reverse")
				.short('r')
				.long("reverse")
				.help("Reverse the kernel module list")
				.action(ArgAction::SetTrue),
		)
		.arg(
			Arg::new("unicode")
				.short('u')
				.long("unicode")
				.help("Show Unicode symbols for the block titles")
				.action(ArgAction::SetTrue),
		)
		.subcommand(
			App::new("sort")
				.about("Sort kernel modules")
				.arg(
					Arg::new("size")
						.short('s')
						.long("size")
						.help("Sort modules by their sizes")
						.action(ArgAction::SetTrue),
				)
				.arg(
					Arg::new("name")
						.short('n')
						.long("name")
						.help("Sort modules by their names")
						.action(ArgAction::SetTrue),
				)
				.arg(
					Arg::new("dependent")
						.short('d')
						.long("dependent")
						.help("Sort modules by their dependent modules")
						.action(ArgAction::SetTrue),
				),
		)
}

#[cfg(test)]
mod tests {
	use super::*;
	#[test]
	fn test_args() {
		get_args().debug_assert();
	}
}

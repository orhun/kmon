use clap::{Arg, ArgAction, ArgMatches, Command as App};
use std::process::Command;

/* Macro for concise initialization of hashmap */
macro_rules! map {
    ($( $key: expr => $val: expr ),*) => {{
         let mut map = ::std::collections::HashMap::new();
         $( map.insert($key, $val); )*
         map
    }}
}

/* Array of the key bindings */
pub const KEY_BINDINGS: &[(&str, &str)] = &[
	("'?', f1", "help"),
	("right/left, h/l", "switch between blocks"),
	("up/down, k/j, alt-k/j", "scroll up/down [selected block]"),
	("pgup/pgdown", "scroll up/down [kernel activities]"),
	("</>", "scroll up/down [module information]"),
	("alt-h/l", "scroll right/left [kernel activities]"),
	("ctrl-t/b, home/end", "scroll to top/bottom [module list]"),
	("alt-e/s", "expand/shrink the selected block"),
	("ctrl-x", "change the block position"),
	("ctrl-l/u, alt-c", "clear the kernel ring buffer"),
	("d, alt-d", "show the dependent modules"),
	("1..9", "jump to the dependent module"),
	("\\, tab, backtab", "show the next kernel information"),
	("/, s, enter", "search a kernel module"),
	("+, i, insert", "load a kernel module"),
	("-, u, backspace", "unload the kernel module"),
	("x, b, delete", "blacklist the kernel module"),
	("ctrl-r, alt-r", "reload the kernel module"),
	("m, o", "show the options menu"),
	("y/n", "execute/cancel the command"),
	("c/v", "copy/paste"),
	("r, f5", "refresh"),
	("q, ctrl-c/d, esc", "quit"),
];

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
 * @return ArgMatches
 */
pub fn parse_args() -> ArgMatches {
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
		.get_matches()
}

/**
 * Execute a operating system command and return its output.
 *
 * @param  cmd
 * @param  cmd_args
 * @return Result
 */
pub fn exec_cmd(cmd: &str, cmd_args: &[&str]) -> Result<String, String> {
	match Command::new(cmd).args(cmd_args).output() {
		Ok(output) => {
			if output.status.success() {
				Ok(String::from_utf8(output.stdout)
					.expect("not UTF-8")
					.trim_end()
					.to_string())
			} else {
				Err(String::from_utf8(output.stderr)
					.expect("not UTF-8")
					.trim_end()
					.to_string())
			}
		}
		Err(e) => Err(e.to_string()),
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	#[test]
	fn test_exec_cmd() {
		assert_eq!("test", exec_cmd("printf", &["test"]).unwrap());
		assert_eq!(
			"true",
			exec_cmd("sh", &["-c", "test 10 -eq 10 && echo 'true'"]).unwrap()
		);
		assert_eq!(
			"err",
			exec_cmd("cat", &["-x"]).unwrap_or(String::from("err"))
		);
	}
}

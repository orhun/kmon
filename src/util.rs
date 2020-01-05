use clap::{App, Arg, SubCommand};
use std::io::{stdout, Error, Write};
use std::process::Command;
use termion::cursor::Goto;

/* Macro for concise initialization of hashmap */
macro_rules! map {
    ($( $key: expr => $val: expr ),*) => {{
         let mut map = ::std::collections::HashMap::new();
         $( map.insert($key, $val); )*
         map
    }}
}

/**
 * Parse command line arguments using clap.
 *
 * @param  version
 * @return ArgMatches
 */
pub fn parse_args(version: &str) -> clap::ArgMatches<'static> {
	App::new("kmon")
		.version(version)
		.author("orhun <orhunparmaksiz@gmail.com>")
		.about("<TODO>")
		.arg(
			Arg::with_name("color")
				.short("c")
				.long("color")
				.value_name("COLOR")
				.help("Set the main color using hex or color name")
				.takes_value(true),
		)
		.arg(
			Arg::with_name("rate")
				.short("t")
				.long("tickrate")
				.value_name("MS")
				.help("Set the refresh rate of the terminal")
				.takes_value(true),
		)
		.arg(
			Arg::with_name("reverse")
				.short("r")
				.long("reverse")
				.help("Reverse the kernel module list"),
		)
		.subcommand(
			SubCommand::with_name("sort")
				.about("Sort kernel modules")
				.arg(
					Arg::with_name("size")
						.short("s")
						.long("size")
						.help("Sort modules by their sizes"),
				)
				.arg(
					Arg::with_name("name")
						.short("n")
						.long("name")
						.help("Sort modules by their names"),
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

/**
 * Set cursor position in terminal.
 *
 * @param  out
 * @param  x
 * @param  y
 * @return Result
 */
pub fn set_cursor_pos<W>(mut out: W, x: u16, y: u16) -> Result<(), Error>
where
	W: Write,
{
	write!(out, "{}", Goto(x, y))?;
	stdout().flush().ok();
	Ok(())
}

#[cfg(test)]
mod tests {
	use super::*;
	#[test]
	fn test_parse_args() {
		let matches = parse_args("0");
		assert_eq!(0, matches.args.len());
		assert_eq!(true, matches.usage.unwrap().lines().count() > 1);
	}
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
	#[test]
	fn test_set_cursor_pos() -> Result<(), Error> {
		let w = Vec::new();
		set_cursor_pos(w, 1, 1)?;
		Ok(())
	}
}

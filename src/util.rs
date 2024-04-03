use std::error::Error;
use std::io::{self, Write};
use std::panic;
use std::process::Command;
use termion::raw::IntoRawMode;

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
 * Sets up the panic hook for the terminal.
 *
 * See <https://ratatui.rs/how-to/develop-apps/panic-hooks/#termion>
 *
 * @return Result
 */
pub fn setup_panic_hook() -> Result<(), Box<dyn Error>> {
	let raw_output = io::stdout().into_raw_mode()?;
	raw_output.suspend_raw_mode()?;

	let panic_hook = panic::take_hook();
	panic::set_hook(Box::new(move |panic| {
		let panic_cleanup = || -> Result<(), Box<dyn Error>> {
			let mut output = io::stdout();
			write!(
				output,
				"{}{}{}",
				termion::clear::All,
				termion::screen::ToMainScreen,
				termion::cursor::Show
			)?;
			raw_output.suspend_raw_mode()?;
			output.flush()?;
			Ok(())
		};
		panic_cleanup().expect("failed to clean up for panic");
		panic_hook(panic);
	}));

	Ok(())
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

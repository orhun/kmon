use std::env;
use std::io::{self, Write};
use std::process::{Command, ExitStatus, Stdio};

/// Text clipboard backed by the helpers for the selected display server.
pub(crate) struct Clipboard {
	paste: (&'static str, &'static [&'static str]),
	copy: (&'static str, &'static [&'static str]),
}

impl Clipboard {
	pub(crate) fn new() -> Option<Self> {
		let has_display =
			|name| env::var_os(name).is_some_and(|value| !value.is_empty());
		Self::for_session(
			env::var("XDG_SESSION_TYPE").ok().as_deref(),
			has_display("WAYLAND_DISPLAY"),
			has_display("DISPLAY"),
		)
	}

	fn for_session(session: Option<&str>, wayland: bool, x11: bool) -> Option<Self> {
		match session {
			Some("wayland") => Self::for_display(true, x11),
			Some("x11") => Self::for_display(false, x11),
			_ => Self::for_display(wayland, x11),
		}
	}

	fn for_display(wayland: bool, x11: bool) -> Option<Self> {
		match (wayland, x11) {
			#[cfg(feature = "wayland")]
			(true, _) => Some(Self {
				paste: ("wl-paste", &["--no-newline"]),
				copy: ("wl-copy", &["--type", "text/plain;charset=utf-8"]),
			}),
			#[cfg(feature = "x11")]
			(_, true) => Some(Self {
				paste: ("xclip", &["-selection", "clipboard", "-out"]),
				copy: ("xclip", &["-selection", "clipboard", "-in"]),
			}),
			_ => None,
		}
	}

	pub(crate) fn get_contents(&mut self) -> io::Result<String> {
		read(Command::new(self.paste.0).args(self.paste.1))
	}

	pub(crate) fn set_contents(&mut self, contents: String) -> io::Result<()> {
		write(Command::new(self.copy.0).args(self.copy.1), &contents)
	}
}

fn read(command: &mut Command) -> io::Result<String> {
	let output = command.output()?;
	check_status(output.status)?;
	String::from_utf8(output.stdout)
		.map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))
}

fn write(command: &mut Command, contents: &str) -> io::Result<()> {
	let mut child = command
		.stdin(Stdio::piped())
		.stdout(Stdio::null())
		.spawn()?;
	let written = child
		.stdin
		.take()
		.expect("piped stdin")
		.write_all(contents.as_bytes());
	let status = child.wait()?;
	written?;
	check_status(status)
}

fn check_status(status: ExitStatus) -> io::Result<()> {
	if status.success() {
		Ok(())
	} else {
		Err(io::Error::other(format!(
			"clipboard command failed: {status}"
		)))
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	fn shell(script: &str) -> Command {
		let mut command = Command::new("sh");
		command.args(["-c", script]);
		command
	}

	#[test]
	fn select_only_enabled_backends() {
		assert!(Clipboard::for_display(false, false).is_none());
		let wayland = Clipboard::for_display(true, false);
		assert_eq!(wayland.is_some(), cfg!(feature = "wayland"));
		if let Some(clipboard) = wayland {
			assert_eq!(clipboard.paste, ("wl-paste", &["--no-newline"][..]));
		}
		let x11 = Clipboard::for_display(false, true);
		assert_eq!(x11.is_some(), cfg!(feature = "x11"));
		if let Some(clipboard) = x11 {
			assert_eq!(clipboard.paste.0, "xclip");
		}
		let mixed = Clipboard::for_display(true, true);
		assert_eq!(
			mixed.is_some(),
			cfg!(any(feature = "wayland", feature = "x11"))
		);
		if let Some(clipboard) = mixed {
			assert_eq!(
				clipboard.paste.0,
				if cfg!(feature = "wayland") {
					"wl-paste"
				} else {
					"xclip"
				}
			);
		}
	}

	#[test]
	fn session_hints_preserve_enabled_xwayland_fallback() {
		assert!(Clipboard::for_session(Some("x11"), true, false).is_none());
		assert_eq!(
			Clipboard::for_session(Some("x11"), true, true).is_some(),
			cfg!(feature = "x11")
		);
		assert_eq!(
			Clipboard::for_session(Some("wayland"), false, false).is_some(),
			cfg!(feature = "wayland")
		);
		assert_eq!(
			Clipboard::for_session(Some("wayland"), false, true).is_some(),
			cfg!(any(feature = "wayland", feature = "x11"))
		);
		assert!(Clipboard::for_session(Some("tty"), false, false).is_none());
		assert_eq!(
			Clipboard::for_session(None, true, false).is_some(),
			cfg!(feature = "wayland")
		);
	}

	#[test]
	fn paste_preserves_utf8_and_trailing_newlines() {
		assert_eq!(
			read(&mut shell("printf 'text λ\\n\\n'")).unwrap(),
			"text λ\n\n"
		);
		assert_eq!(read(&mut shell("printf ''")).unwrap(), "");
	}

	#[test]
	fn paste_reports_command_failure() {
		assert_eq!(
			read(&mut shell("printf partial; exit 7"))
				.unwrap_err()
				.kind(),
			io::ErrorKind::Other
		);
	}

	#[test]
	fn paste_rejects_invalid_utf8() {
		assert_eq!(
			read(&mut shell("printf '\\377'")).unwrap_err().kind(),
			io::ErrorKind::InvalidData
		);
	}

	#[test]
	fn copy_writes_exact_bytes_and_closes_stdin() {
		write(
			&mut shell(
				"test \"$(od -An -tx1 | tr -d ' \\n')\" = '7465787420cebb0a0a'",
			),
			"text λ\n\n",
		)
		.unwrap();
		write(&mut shell("test \"$(wc -c)\" -eq 0"), "").unwrap();
	}

	#[test]
	fn copy_reports_command_failure() {
		assert_eq!(
			write(&mut shell("cat >/dev/null; exit 7"), "text")
				.unwrap_err()
				.kind(),
			io::ErrorKind::Other
		);
	}

	#[test]
	fn copy_reports_broken_pipe() {
		assert!(
			write(&mut shell("exec 0<&-; exit 0"), &"x".repeat(1024 * 1024))
				.is_err()
		);
	}

	#[test]
	fn missing_helpers_return_an_error() {
		assert_eq!(
			read(&mut Command::new("/nonexistent-kmon-clipboard-helper"))
				.unwrap_err()
				.kind(),
			io::ErrorKind::NotFound
		);
		assert_eq!(
			write(
				&mut Command::new("/nonexistent-kmon-clipboard-helper"),
				"text"
			)
			.unwrap_err()
			.kind(),
			io::ErrorKind::NotFound
		);
	}
}

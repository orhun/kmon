use std::io::{self, Write};
use std::process::Command;
use clap::{App};
use tui::Terminal;
use tui::backend::TermionBackend;
use termion::raw::IntoRawMode;

const VERSION: &'static str = "0.1.0"; /* Version */

/**
 * Execute a operating system command and return its output.
 *
 * @param  cmd
 * @param  cmd_args
 * @return result
 */
fn exec_cmd(cmd: &str, cmd_args: &[&str]) -> Result<String, String> {
    let output = Command::new(cmd).args(cmd_args)
        .output().expect("failed to execute command");
    /* Write error output to stderr stream. */
    io::stderr().write_all(&output.stderr).unwrap();
    if output.status.success() {
        Ok(String::from_utf8(output.stdout).expect("not UTF-8"))
    } else {
        Err(format!("{} {}", cmd, cmd_args.join(" ")))
    }
}

/**
 * Parse command line arguments using 'clap'.
 */
fn parse_args() {
    let _matches = App::new("kmon")
        .version(VERSION).get_matches();
}

/**
 * Create a terminal instance using termion as backend.
 *
 * @return result
 */
fn create_term() -> Result<(), io::Error> {
    let stdout = io::stdout().into_raw_mode()?;
    let backend = TermionBackend::new(stdout);
    let mut _terminal = Terminal::new(backend)?;
    Ok(())
}

/**
 * Entry point.
 */
fn main() {
    parse_args();
    println!("{}", exec_cmd("sh", &["-c", "echo 'x'"]).unwrap());
    create_term().expect("failed to create terminal");
}

/**
 * Unit test module.
 */
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_exec_cmd() {
        assert_eq!("test", exec_cmd("printf", &["test"]).unwrap());
        assert_eq!("true\n", exec_cmd("sh", &["-c",
            "test 10 -eq 10 && echo 'true'"]).unwrap());
    }
}

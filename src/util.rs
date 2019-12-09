use clap::App;
use clap::Arg;
use clap::SubCommand;
use std::io::{stdout, Error, Write};
use std::process::Command;
use termion::cursor::Goto;

/* Supported directions of scrolling */
pub enum ScrollDirection {
    Up,
    Down,
    Top,
    Bottom,
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
        .arg(
            Arg::with_name("rate")
                .short("r")
                .long("rate")
                .value_name("MS")
                .help("Refresh rate of the terminal")
                .takes_value(true),
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
    let output = Command::new(cmd)
        .args(cmd_args)
        .output()
        .expect("failed to execute command");
    if output.status.success() {
        Ok((String::from_utf8(output.stdout).expect("not UTF-8"))
            .trim_end()
            .to_string())
    } else {
        Err(format!("{} {}", cmd, cmd_args.join(" ")))
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
    }
}

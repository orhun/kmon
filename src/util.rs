use clap::App;
use clap::Arg;
use clap::SubCommand;
use std::io::{self, Write};
use std::process::Command;

/**
 * Parse command line arguments using 'clap'.
 *
 * @return ArgMatches
 */
pub fn parse_args(version: &str) -> clap::ArgMatches<'static> {
    App::new("kmon")
        .version(version)
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
    /* Write error output to stderr stream. */
    io::stderr().write_all(&output.stderr).unwrap();
    if output.status.success() {
        Ok((String::from_utf8(output.stdout).expect("not UTF-8"))
            .trim_end()
            .to_string())
    } else {
        Err(format!("{} {}", cmd, cmd_args.join(" ")))
    }
}

/**
 * Unit tests.
 */
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_parse_args() {
        let matches = parse_args();
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

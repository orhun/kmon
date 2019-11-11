use clap::{App};
use std::process::Command;
use std::io::{self, Write};

const VERSION: &'static str = "0.1.0"; /* Version */

/**
 * Execute a operating system command and return its output.
 *
 * @param  cmd
 * @param  cmd_args
 * @return output
 */
fn exec_cmd(cmd: &str, cmd_args: &[&str]) -> String {
    let output = Command::new(cmd).args(cmd_args)
        .output().expect("failed to execute command");
    /* Write error output to stderr stream. */
    io::stderr().write_all(&output.stderr).unwrap();
    if output.status.success() {
        return String::from_utf8(output.stdout).expect("not UTF-8");
    } else {
        panic!("{} {}", cmd, cmd_args.join(" "));
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
 * Entry point.
 */
fn main() {
    parse_args();
    println!("{}", exec_cmd("sh", &["-c", "echo 'x'"]));
}

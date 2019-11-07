extern crate clap;
use clap::{Arg, App};
use std::process::Command;
use std::io::{self, Write};

const VERSION: &'static str = "0.1.0";

fn exec_cmd(cmd: &str, cmd_args: &[&str]) -> String {
    let output = Command::new(cmd)
                        .args(cmd_args)
                        .output()
                        .expect("failed to execute command");
    io::stderr().write_all(&output.stderr).unwrap();
    if output.status.success() {
        return String::from_utf8(output.stdout).expect("Not UTF-8");
    } else {
        return String::from("woo");
    }
}

fn parse_args() {
    let _matches = App::new("kmon")
                    .version(VERSION)
                    .get_matches();
}

fn main() {
    parse_args();
    println!("{}", exec_cmd("ls", &["-a", "-x"]));
}

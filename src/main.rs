extern crate clap;
use clap::{Arg, App};
use std::process::Command;
use std::io::{self, Write};

const VERSION: &'static str = "0.1.0";

fn exec_cmd() {
    let output = Command::new("ls")
                        .arg("-a")
                        .output()
                        .expect("failed to execute command");
    println!("status: {}", output.status);
    io::stdout().write_all(&output.stdout).unwrap();
    io::stderr().write_all(&output.stderr).unwrap();
}

fn parse_args() {
    let matches = App::new("kmon")
                    .version(VERSION)
                    .get_matches();
}

fn main() {
    parse_args();
    exec_cmd();
}

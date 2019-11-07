extern crate clap;
use clap::{Arg, App};

const VERSION: &'static str = "0.1.0";

fn parse_args() {
    let matches = App::new("kmon")
                    .version(VERSION)
                    .get_matches();
}

fn main() {
    parse_args();
}

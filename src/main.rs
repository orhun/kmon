extern crate clap;
use clap::{Arg, App};

const VERSION: &'static str = "0.1.0";

fn parse_args() -> bool {
    let matches = App::new("kmon")
                    .version(VERSION)
                    .get_matches();
    return true;
}

fn main() {
    parse_args();
}

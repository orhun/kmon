const VERSION: &'static str = "0.1.0";

fn parse_args() -> bool {
    println!("kmon v{}", VERSION);
    return true;
}

fn main() {
    parse_cli_args();
}

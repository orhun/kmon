const VERSION: &'static str = "0.1.0";
const PKG_VERSION:Option<&'static str> = option_env!("CARGO_PKG_VERSION");

fn main() {
    println!("kmon v{}", PKG_VERSION.unwrap_or(VERSION));
}

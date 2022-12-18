#[path = "src/args.rs"]
mod args;

use clap::ValueEnum;
use clap_complete::{self, Shell};
use clap_mangen::Man;
use std::env;
use std::error::Error;
use std::fs::{self, File};
use std::io::Error as IoError;
use std::path::{Path, PathBuf};

fn build_shell_completions(out_dir: &Path) -> Result<(), IoError> {
	fs::create_dir_all(out_dir)?;
	let mut app = args::get_args();
	let shells = Shell::value_variants();
	for shell in shells {
		clap_complete::generate_to(
			*shell,
			&mut app,
			env!("CARGO_PKG_NAME"),
			out_dir,
		)?;
	}
	Ok(())
}

fn build_manpage(out_dir: &Path) -> Result<(), IoError> {
	fs::create_dir_all(out_dir)?;
	let app = args::get_args();
	let file = out_dir.join(format!("{}.8", env!("CARGO_PKG_NAME")));
	let mut file = File::create(file)?;
	Man::new(app).render(&mut file)?;
	Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
	println!("cargo:rerun-if-changed=src/args.rs");
	let out_dir = match env::var_os("OUT_DIR").map(PathBuf::from) {
		None => return Ok(()),
		Some(v) => v
			.ancestors()
			.nth(4)
			.expect("failed to determine out dir")
			.to_owned(),
	};
	build_manpage(&out_dir.join("man"))?;
	build_shell_completions(&out_dir.join("completions"))?;
	Ok(())
}

use kmon::args;
use kmon::event::Events;
use kmon::kernel::Kernel;
use ratatui::backend::TermionBackend;
use ratatui::Terminal;
use std::error::Error;
use std::io::stdout;
use std::io::{self, Write};
use std::panic;
use termion::input::MouseTerminal;
use termion::raw::IntoRawMode;
use termion::screen::IntoAlternateScreen;

/**
 * Entry point.
 *
 * @return Result
 */
fn main() -> Result<(), Box<dyn Error>> {

	let raw_output = io::stdout().into_raw_mode()?;
	raw_output.suspend_raw_mode()?;

	let panic_hook = panic::take_hook();

	panic::set_hook(Box::new (move |panic| {
		let panic_cleanup = || -> Result<(), Box<dyn Error>> {
			let mut output = io::stdout();

			write!(
				output,
				"{}{}{}",
				termion::clear::All,
				termion::screen::ToMainScreen,
				termion::cursor::Show
			)?;
			raw_output.suspend_raw_mode()?;
			output.flush()?;
			Ok(())
		};

		panic_cleanup().expect("Failed to cleanup after panic");
		panic_hook(panic);
	}));

	let args = args::get_args().get_matches();
	let kernel = Kernel::new(&args);
	let events = Events::new(
		args.get_one::<String>("rate")
			.unwrap()
			.parse::<u64>()
			.unwrap_or(250),
		&kernel.logs,
	);
	if !cfg!(test) {
		let stdout = stdout().into_raw_mode()?.into_alternate_screen()?;
		let stdout = MouseTerminal::from(stdout);
		let backend = TermionBackend::new(stdout);
		kmon::start_tui(Terminal::new(backend)?, kernel, &events)
	} else {
		Ok(())
	}
}

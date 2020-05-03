use crate::kernel::log::KernelLogs;
use std::io;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use termion::event::Key;
use termion::input::TermRead;

/* Terminal event methods */
pub enum Event<I> {
	Input(I),
	Kernel(String),
	Tick,
}

/* Terminal events */
#[allow(dead_code)]
pub struct Events {
	pub tx: mpsc::Sender<Event<Key>>,
	pub rx: mpsc::Receiver<Event<Key>>,
	input_handler: thread::JoinHandle<()>,
	kernel_handler: thread::JoinHandle<()>,
	tick_handler: thread::JoinHandle<()>,
}

impl Events {
	/**
	 * Create a new events instance.
	 *
	 * @param  refresh_rate
	 * @param  kernel_logs
	 * @return Events
	 */
	pub fn new(refresh_rate: u64, kernel_logs: &KernelLogs) -> Self {
		/* Convert refresh rate to Duration from milliseconds. */
		let refresh_rate = Duration::from_millis(refresh_rate);
		/* Create a new asynchronous channel. */
		let (tx, rx) = mpsc::channel();
		/* Handle inputs using stdin stream and sender of the channel. */
		let input_handler = {
			let tx = tx.clone();
			thread::spawn(move || {
				let stdin = io::stdin();
				for evt in stdin.keys() {
					if let Ok(key) = evt {
						tx.send(Event::Input(key)).unwrap();
					}
				}
			})
		};
		/* Handle kernel logs using 'dmesg' output. */
		let kernel_handler = {
			let tx = tx.clone();
			let mut kernel_logs = kernel_logs.clone();
			thread::spawn(move || loop {
				if kernel_logs.update() {
					tx.send(Event::Kernel(kernel_logs.output.to_string()))
						.unwrap_or_default();
				}
				thread::sleep(refresh_rate * 10);
			})
		};
		/* Create a loop for handling events. */
		let tick_handler = {
			let tx = tx.clone();
			thread::spawn(move || loop {
				tx.send(Event::Tick).unwrap_or_default();
				thread::sleep(refresh_rate);
			})
		};
		/* Return events. */
		Self {
			tx,
			rx,
			input_handler,
			kernel_handler,
			tick_handler,
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::error::Error;
	#[test]
	fn test_events() -> Result<(), Box<dyn Error>> {
		let kernel_logs = KernelLogs::default();
		let events = Events::new(100, &kernel_logs);
		let mut i = 0;
		loop {
			let tx = events.tx.clone();
			thread::spawn(move || {
				match tx.send(Event::Input(Key::Char(
					std::char::from_digit(i, 10).unwrap_or('x'),
				))) {
					_ => {}
				};
			});
			i += 1;
			match events.rx.recv()? {
				Event::Input(v) => {
					if v == Key::Char('9') {
						break;
					}
				}
				Event::Tick => thread::sleep(Duration::from_millis(100)),
				Event::Kernel(log) => assert_ne!(true, log.is_empty()),
			}
		}
		Ok(())
	}
}

mod app;
mod event;
mod kernel;
#[macro_use]
mod util;
mod style;
use app::{App, Block, InputMode, ScrollDirection};
use enum_unitary::{Bounded, EnumUnitary};
use event::{Event, Events};
use kernel::cmd::ModuleCommand;
use kernel::Kernel;
use std::io::stdout;
use termion::event::Key;
use termion::input::MouseTerminal;
use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;
use tui::backend::{Backend, TermionBackend};
use tui::layout::{Constraint, Direction, Layout};
use tui::Terminal;
use unicode_width::UnicodeWidthStr;

/**
 * Configure the terminal and draw its widgets.
 *
 * @param  Terminal
 * @param  Kernel
 * @param  Events
 * @return Result
 */
fn start_tui<B>(
	mut terminal: Terminal<B>,
	mut kernel: Kernel,
	events: &Events,
) -> Result<(), failure::Error>
where
	B: Backend,
{
	/* Configure the application. */
	let mut app = App::new(Block::ModuleTable, kernel.modules.style.clone());
	/* Draw terminal and render the widgets. */
	terminal.hide_cursor()?;
	loop {
		terminal.draw(|mut f| {
			let chunks = Layout::default()
				.direction(Direction::Vertical)
				.constraints(
					[Constraint::Percentage(75), Constraint::Percentage(25)]
						.as_ref(),
				)
				.split(f.size());
			{
				let chunks = Layout::default()
					.direction(Direction::Horizontal)
					.constraints(
						[Constraint::Percentage(60), Constraint::Percentage(40)]
							.as_ref(),
					)
					.split(chunks[0]);
				{
					let chunks = Layout::default()
						.direction(Direction::Vertical)
						.constraints(
							[Constraint::Length(3), Constraint::Percentage(100)]
								.as_ref(),
						)
						.split(chunks[0]);
					{
						let chunks = Layout::default()
							.direction(Direction::Horizontal)
							.constraints(
								[
									Constraint::Percentage(60),
									Constraint::Percentage(40),
								]
								.as_ref(),
							)
							.split(chunks[0]);
						app.draw_user_input(&mut f, chunks[0], &events.tx);
						app.draw_kernel_info(
							&mut f,
							chunks[1],
							&kernel.info.current_info,
						);
					}
					app.draw_kernel_modules(&mut f, chunks[1], &mut kernel.modules);
				}
				app.draw_module_info(&mut f, chunks[1], &mut kernel.modules);
			}
			app.draw_kernel_activities(&mut f, chunks[1], &mut kernel.logs);
		})?;
		/* Set cursor position if the input mode flag is set. */
		if !app.input_mode.is_none() {
			terminal.set_cursor(1 + app.input_query.width() as u16, 1)?;
		}
		/* Handle terminal events. */
		match events.rx.recv()? {
			/* Key input events. */
			Event::Input(input) => {
				if app.input_mode.is_none() {
					/* Default input mode. */
					match input {
						/* Quit. */
						Key::Char('q')
						| Key::Char('Q')
						| Key::Ctrl('c')
						| Key::Ctrl('d')
						| Key::Esc => {
							break;
						}
						/* Refresh. */
						Key::Char('r') | Key::Char('R') | Key::F(5) => {
							app.refresh();
							kernel.refresh();
						}
						/* Show help message. */
						Key::Char('?') | Key::F(1) => {
							app.show_help_message(&mut kernel.modules);
						}
						/* Scroll the selected block up. */
						Key::Up
						| Key::Char('k')
						| Key::Char('K')
						| Key::Alt('k')
						| Key::Alt('K') => match app.selected_block {
							Block::ModuleTable => {
								kernel.modules.scroll_list(ScrollDirection::Up)
							}
							Block::ModuleInfo => kernel.modules.scroll_mod_info(
								ScrollDirection::Up,
								input == Key::Alt('k') || input == Key::Alt('K'),
							),
							Block::Activities => {
								kernel.logs.scroll(
									ScrollDirection::Up,
									input == Key::Alt('k') || input == Key::Alt('K'),
								);
							}
							_ => {}
						},
						/* Scroll the selected block down. */
						Key::Down
						| Key::Char('j')
						| Key::Char('J')
						| Key::Alt('j')
						| Key::Alt('J') => match app.selected_block {
							Block::ModuleTable => {
								kernel.modules.scroll_list(ScrollDirection::Down)
							}
							Block::ModuleInfo => kernel.modules.scroll_mod_info(
								ScrollDirection::Down,
								input == Key::Alt('j') || input == Key::Alt('J'),
							),
							Block::Activities => {
								kernel.logs.scroll(
									ScrollDirection::Down,
									input == Key::Alt('j') || input == Key::Alt('J'),
								);
							}
							_ => {}
						},
						/* Select the next terminal block. */
						Key::Left | Key::Char('h') | Key::Char('H') => {
							app.selected_block =
								match app.selected_block.prev_variant() {
									Some(v) => v,
									None => Block::max_value(),
								}
						}
						/* Select the previous terminal block. */
						Key::Right | Key::Char('l') | Key::Char('L') => {
							app.selected_block =
								match app.selected_block.next_variant() {
									Some(v) => v,
									None => Block::min_value(),
								}
						}
						/* Scroll to the top of the module list. */
						Key::Ctrl('t') | Key::Home => {
							app.selected_block = Block::ModuleTable;
							kernel.modules.scroll_list(ScrollDirection::Top)
						}
						/* Scroll to the bottom of the module list. */
						Key::Ctrl('b') | Key::End => {
							app.selected_block = Block::ModuleTable;
							kernel.modules.scroll_list(ScrollDirection::Bottom)
						}
						/* Scroll kernel activities up. */
						Key::PageUp => {
							app.selected_block = Block::Activities;
							kernel.logs.scroll(ScrollDirection::Up, false);
						}
						/* Scroll kernel activities down. */
						Key::PageDown => {
							app.selected_block = Block::Activities;
							kernel.logs.scroll(ScrollDirection::Down, false);
						}
						/* Scroll kernel activities left. */
						Key::Alt('h') | Key::Alt('H') => {
							app.selected_block = Block::Activities;
							kernel.logs.scroll(ScrollDirection::Left, false);
						}
						/* Scroll kernel activities right. */
						Key::Alt('l') | Key::Alt('L') => {
							app.selected_block = Block::Activities;
							kernel.logs.scroll(ScrollDirection::Right, false);
						}
						/* Scroll module information up. */
						Key::Char('<') | Key::Alt(' ') => {
							app.selected_block = Block::ModuleInfo;
							kernel
								.modules
								.scroll_mod_info(ScrollDirection::Up, false)
						}
						/* Scroll module information down. */
						Key::Char('>') | Key::Char(' ') => {
							app.selected_block = Block::ModuleInfo;
							kernel
								.modules
								.scroll_mod_info(ScrollDirection::Down, false)
						}
						/* Show the next kernel information. */
						Key::Char('\\') | Key::Char('\t') | Key::BackTab => {
							kernel.info.next();
						}
						/* Clear the kernel ring buffer. */
						Key::Ctrl('l')
						| Key::Ctrl('u')
						| Key::Alt('c')
						| Key::Alt('C') => {
							kernel.modules.set_current_command(
								ModuleCommand::Clear,
								String::new(),
							);
						}
						/* Unload kernel module. */
						Key::Char('u')
						| Key::Char('U')
						| Key::Char('-')
						| Key::Backspace
						| Key::Ctrl('h') => {
							kernel.modules.set_current_command(
								ModuleCommand::Unload,
								String::new(),
							);
						}
						/* Blacklist kernel module. */
						Key::Char('x')
						| Key::Char('X')
						| Key::Char('b')
						| Key::Char('B')
						| Key::Delete => {
							kernel.modules.set_current_command(
								ModuleCommand::Blacklist,
								String::new(),
							);
						}
						/* Reload kernel module. */
						Key::Ctrl('r')
						| Key::Ctrl('R')
						| Key::Alt('r')
						| Key::Alt('R') => {
							kernel.modules.set_current_command(
								ModuleCommand::Reload,
								String::new(),
							);
						}
						/* Execute the current command. */
						Key::Char('y') | Key::Char('Y') => {
							if kernel.modules.execute_command() {
								events
									.tx
									.send(Event::Input(Key::Char('r')))
									.unwrap();
							}
						}
						/* Cancel the execution of current command. */
						Key::Char('n') | Key::Char('N') => {
							if kernel.modules.cancel_execution() {
								app.selected_block = Block::ModuleTable;
							}
						}
						/* Copy the data in selected block to clipboard. */
						Key::Char('c') | Key::Char('C') => {
							app.set_clipboard_contents(match app.selected_block {
								Block::ModuleTable => &kernel.modules.current_name,
								Block::ModuleInfo => {
									&kernel.modules.current_info.raw_text
								}
								Block::Activities => {
									&kernel.logs.selected_output.trim()
								}
								_ => "",
							});
						}
						/* Paste the clipboard contents and switch to search mode. */
						Key::Char('v') | Key::Ctrl('V') | Key::Ctrl('v') => {
							app.input_query += &app.get_clipboard_contents();
							events.tx.send(Event::Input(Key::Char('\n'))).unwrap();
							kernel.modules.index = 0;
						}
						/* User input mode. */
						Key::Char('\n')
						| Key::Char('s')
						| Key::Char('S')
						| Key::Char('i')
						| Key::Char('I')
						| Key::Char('+')
						| Key::Char('/')
						| Key::Insert => {
							app.selected_block = Block::UserInput;
							app.input_mode = match input {
								Key::Char('+')
								| Key::Char('i')
								| Key::Char('I')
								| Key::Insert => InputMode::Load,
								_ => InputMode::Search,
							};
							if input != Key::Char('\n') {
								app.input_query = String::new();
							}
							terminal
								.set_cursor(1 + app.input_query.width() as u16, 1)?;
							terminal.show_cursor()?;
						}
						/* Other character input. */
						Key::Char(v) => {
							/* Check if input is a number except zero. */
							let index = v.to_digit(10).unwrap_or(0);
							/* Show the used module info at given index. */
							if index != 0 && !kernel.modules.list.is_empty() {
								app.selected_block = Block::ModuleTable;
								kernel.modules.show_used_module(index as usize - 1);
							}
						}
						_ => {}
					}
				} else {
					/* User input mode. */
					match input {
						/* Quit with ctrl-d. */
						Key::Ctrl('d') => {
							break;
						}
						/* Switch to the previous input mode. */
						Key::Up => {
							app.input_mode = match app.input_mode.prev_variant() {
								Some(v) => v,
								None => InputMode::max_value(),
							};
							if app.input_mode.is_none() {
								app.input_mode = InputMode::max_value();
							}
							app.input_query = String::new();
						}
						/* Switch to the next input mode. */
						Key::Down => {
							app.input_mode = match app.input_mode.next_variant() {
								Some(v) => v,
								None => {
									InputMode::min_value().next_variant().unwrap()
								}
							};
							app.input_query = String::new();
						}
						/* Copy input query to the clipboard. */
						Key::Ctrl('c') => {
							app.set_clipboard_contents(&app.input_query);
						}
						/* Paste the clipboard contents. */
						Key::Ctrl('v') => {
							app.input_query += &app.get_clipboard_contents();
						}
						/* Exit user input mode. */
						Key::Char('\n')
						| Key::Char('\t')
						| Key::Char('?')
						| Key::F(1)
						| Key::Right
						| Key::Left => {
							/* Select the next eligible block for action. */
							app.selected_block = match input {
								Key::Left => {
									match app.selected_block.prev_variant() {
										Some(v) => v,
										None => Block::max_value(),
									}
								}
								Key::Char('\n') => match app.input_mode {
									InputMode::Load
										if !app.input_query.is_empty() =>
									{
										Block::ModuleInfo
									}
									_ => Block::ModuleTable,
								},
								Key::Char('?') | Key::F(1) => {
									app.show_help_message(&mut kernel.modules);
									app.input_mode = InputMode::None;
									Block::ModuleTable
								}
								_ => Block::ModuleTable,
							};
							/* Show the first modules information if the search mode is set. */
							if app.input_mode == InputMode::Search
								&& kernel.modules.index == 0
							{
								kernel.modules.scroll_list(ScrollDirection::Top);
							/* Load kernel module. */
							} else if app.input_mode == InputMode::Load
								&& !app.input_query.is_empty()
							{
								kernel.modules.set_current_command(
									ModuleCommand::Load,
									app.input_query,
								);
								app.input_query = String::new();
							}
							/* Hide terminal cursor and set the input mode flag. */
							terminal.hide_cursor()?;
							app.input_mode = InputMode::None;
						}
						/* Append character to input query. */
						Key::Char(c) => {
							app.input_query.push(c);
							kernel.modules.index = 0;
						}
						/* Delete the last character from input query. */
						Key::Backspace | Key::Ctrl('h') => {
							app.input_query.pop();
							kernel.modules.index = 0;
						}
						/* Clear the input query. */
						Key::Delete | Key::Ctrl('l') => {
							app.input_query = String::new();
							kernel.modules.index = 0;
						}
						/* Clear the input query and exit user input mode. */
						Key::Esc => {
							events.tx.send(Event::Input(Key::Delete)).unwrap();
							events.tx.send(Event::Input(Key::Char('\n'))).unwrap();
						}
						_ => {}
					}
				}
			}
			/* Kernel events. */
			Event::Kernel(logs) => {
				kernel.logs.output = logs;
			}
			_ => {}
		}
	}
	Ok(())
}

/**
 * Entry point.
 *
 * @return Result
 */
fn main() -> Result<(), failure::Error> {
	let args = util::parse_args();
	let kernel = Kernel::new(&args);
	let events = Events::new(
		args.value_of("rate").unwrap().parse::<u64>().unwrap_or(250),
		&kernel.logs,
	);
	if !cfg!(test) {
		let stdout = stdout().into_raw_mode()?;
		let stdout = MouseTerminal::from(stdout);
		let stdout = AlternateScreen::from(stdout);
		let backend = TermionBackend::new(stdout);
		start_tui(Terminal::new(backend)?, kernel, &events)
	} else {
		Ok(())
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::sync::mpsc::Sender;
	use std::thread;
	use std::time::Duration;
	use tui::backend::TestBackend;
	#[test]
	fn test_tui() -> Result<(), failure::Error> {
		main()?;
		let args = util::parse_args();
		let kernel = Kernel::new(&args);
		let events = Events::new(100, &kernel.logs);
		let tx = events.tx.clone();
		thread::spawn(move || {
			/* Test the general keys. */
			for key in vec![
				Key::Char('?'),
				Key::Ctrl('t'),
				Key::Ctrl('b'),
				Key::Char('x'),
				Key::Char('n'),
				Key::Ctrl('l'),
				Key::Char('u'),
				Key::Ctrl('r'),
				Key::Char('y'),
				Key::PageUp,
				Key::PageDown,
				Key::Alt('l'),
				Key::Alt('h'),
				Key::Char('<'),
				Key::Char('>'),
				Key::Char('\t'),
			] {
				send_key(&tx, key);
			}
			send_key(&tx, Key::Char('r'));
			/* Test the switch keys. */
			for arrow_key in vec![Key::Right, Key::Left] {
				for selected_key in vec![arrow_key; Block::count()] {
					send_key(&tx, selected_key);
					for key in vec![
						Key::Up,
						Key::Down,
						Key::Down,
						Key::Up,
						Key::Char('c'),
						Key::Char('~'),
						Key::Char('1'),
					] {
						send_key(&tx, key);
					}
				}
			}
			/* Test the input mode keys. */
			for key in vec![
				Key::Char('v'),
				Key::Delete,
				Key::Char('~'),
				Key::Backspace,
				Key::Ctrl('c'),
				Key::Ctrl('v'),
				Key::Char('a'),
				Key::Char('\n'),
				Key::Char('\n'),
				Key::Char('?'),
				Key::Char('\n'),
				Key::Esc,
				Key::Char('i'),
				Key::Char('x'),
				Key::Char('\n'),
			] {
				send_key(&tx, key);
			}
			/* Exit. */
			send_key(&tx, Key::Esc)
		});
		start_tui(Terminal::new(TestBackend::new(20, 10))?, kernel, &events)
	}
	/**
	 * Try to send a key event until Sender succeeds.
	 *
	 * @param Sender
	 * @param Key
	 */
	fn send_key(tx: &Sender<Event<Key>>, key: Key) {
		let mut x = true;
		while x {
			x = tx.send(Event::Input(key)).is_err();
			thread::sleep(Duration::from_millis(10));
		}
	}
}

#![allow(clippy::tabs_in_doc_comments)]

pub mod app;
pub mod event;
pub mod kernel;
pub mod widgets;
#[macro_use]
pub mod util;
pub mod args;
pub mod style;

use crate::app::{App, Block, InputMode, ScrollDirection};
use crate::kernel::cmd::ModuleCommand;
use crate::kernel::Kernel;
use enum_iterator::Sequence;
use event::{Event, Events};
use ratatui::backend::Backend;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::Terminal;
use std::error::Error;
use termion::event::Key;
use unicode_width::UnicodeWidthStr;

/**
 * Configure the terminal and draw its widgets.
 *
 * @param  Terminal
 * @param  Kernel
 * @param  Events
 * @return Result
 */
pub fn start_tui<B>(
	mut terminal: Terminal<B>,
	mut kernel: Kernel,
	events: &Events,
) -> Result<(), Box<dyn Error>>
where
	B: Backend,
{
	/* Configure the application. */
	let mut app = App::new(Block::ModuleTable, kernel.modules.style.clone());
	/* Draw terminal and render the widgets. */
	loop {
		terminal.draw(|frame| {
			let chunks = Layout::default()
				.direction(Direction::Vertical)
				.constraints(
					[
						Constraint::Percentage(100 - app.block_size.activities),
						Constraint::Percentage(app.block_size.activities),
					]
					.as_ref(),
				)
				.split(frame.size());
			{
				let chunks = Layout::default()
					.direction(Direction::Horizontal)
					.constraints(
						[
							Constraint::Percentage(100 - app.block_size.info),
							Constraint::Percentage(app.block_size.info),
						]
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
									Constraint::Percentage(app.block_size.input),
									Constraint::Percentage(
										100 - app.block_size.input,
									),
								]
								.as_ref(),
							)
							.split(chunks[0]);
						app.draw_user_input(frame, chunks[0], &events.tx);
						app.draw_kernel_info(
							frame,
							chunks[1],
							&kernel.info.current_info,
						);
					}
					if app.block_size.info != 100 {
						app.draw_dynamic_block(frame, chunks[1], &mut kernel);
					} else {
						app.block_index += 1;
					}
				}
				app.draw_dynamic_block(frame, chunks[1], &mut kernel);
			}
			app.draw_dynamic_block(frame, chunks[1], &mut kernel);
			if !app.input_mode.is_none() {
				frame.set_cursor(1 + app.input_query.width() as u16, 1);
			}
		})?;
		/* Handle terminal events. */
		match events.rx.recv()? {
			/* Key input events. */
			Event::Input(input) => {
				let mut hide_options = true;
				if app.input_mode.is_none() {
					/* Default input mode. */
					match input {
						/* Quit. */
						Key::Char('q')
						| Key::Char('Q')
						| Key::Ctrl('c')
						| Key::Ctrl('d')
						| Key::Esc => {
							if app.show_options {
								app.show_options = false;
							} else {
								break;
							}
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
						Key::Char('m') | Key::Char('o') => {
							app.show_options = true;
							hide_options = false;
						}
						/* Scroll the selected block up. */
						Key::Up
						| Key::Char('k')
						| Key::Char('K')
						| Key::Alt('k')
						| Key::Alt('K') => {
							if app.show_options {
								app.options.previous();
								continue;
							} else {
								app.options.state.select(Some(0));
							}
							match app.selected_block {
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
										input == Key::Alt('k')
											|| input == Key::Alt('K'),
									);
								}
								_ => {}
							}
						}
						/* Scroll the selected block down. */
						Key::Down
						| Key::Char('j')
						| Key::Char('J')
						| Key::Alt('j')
						| Key::Alt('J') => {
							if app.show_options {
								app.options.next();
								continue;
							} else {
								app.options.state.select(Some(0));
							}
							match app.selected_block {
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
										input == Key::Alt('j')
											|| input == Key::Alt('J'),
									);
								}
								_ => {}
							}
						}
						/* Select the next terminal block. */
						Key::Left | Key::Char('h') | Key::Char('H') => {
							app.selected_block = match app.selected_block.previous()
							{
								Some(v) => v,
								None => Block::last().unwrap(),
							}
						}
						/* Select the previous terminal block. */
						Key::Right | Key::Char('l') | Key::Char('L') => {
							app.selected_block = match app.selected_block.next() {
								Some(v) => v,
								None => Block::first().unwrap(),
							}
						}
						/* Expand the selected block. */
						Key::Alt('e') => {
							let block_size = app.block_size();
							if *block_size < 95 {
								*block_size += 5;
							} else {
								*block_size = 100;
							}
						}
						/* Shrink the selected block. */
						Key::Alt('s') => {
							let block_size = app.block_size();
							*block_size =
								(*block_size).checked_sub(5).unwrap_or_default()
						}
						/* Change the block position. */
						Key::Ctrl('x') => {
							if app.block_index == 2 {
								app.block_index = 0;
							} else {
								app.block_index += 1;
							}
						}
						/* Scroll to the top of the module list. */
						Key::Ctrl('t') | Key::Home => {
							app.options.state.select(Some(0));
							app.selected_block = Block::ModuleTable;
							kernel.modules.scroll_list(ScrollDirection::Top)
						}
						/* Scroll to the bottom of the module list. */
						Key::Ctrl('b') | Key::End => {
							app.options.state.select(Some(0));
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
						/* Display the dependent modules. */
						Key::Char('d') | Key::Alt('d') => {
							app.show_dependent_modules(&mut kernel.modules);
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
									kernel.logs.selected_output.trim()
								}
								_ => "",
							});
						}
						/* Paste the clipboard contents and switch to search mode. */
						Key::Char('v') | Key::Ctrl('V') | Key::Ctrl('v') => {
							let clipboard_contents = app.get_clipboard_contents();
							app.input_query += &clipboard_contents;
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
							if input == Key::Char('\n') && app.show_options {
								if let Ok(command) = ModuleCommand::try_from(
									app.options
										.selected()
										.map(|(v, _)| v.to_string())
										.unwrap_or_default(),
								) {
									if command == ModuleCommand::Load {
										events
											.tx
											.send(Event::Input(Key::Char('+')))
											.unwrap();
									} else {
										kernel.modules.set_current_command(
											command,
											String::new(),
										);
									}
								} else {
									match app
										.options
										.selected()
										.map(|(v, _)| v.as_ref())
									{
										Some("dependent") => {
											app.show_dependent_modules(
												&mut kernel.modules,
											);
										}
										Some("copy") => app.set_clipboard_contents(
											&kernel.modules.current_name,
										),
										_ => {}
									}
								}
							} else {
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
							}
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
							app.input_mode = match app.input_mode.previous() {
								Some(v) => v,
								None => InputMode::last().unwrap(),
							};
							if app.input_mode.is_none() {
								app.input_mode = InputMode::last().unwrap();
							}
							app.input_query = String::new();
						}
						/* Switch to the next input mode. */
						Key::Down => {
							app.input_mode = match app.input_mode.next() {
								Some(v) => v,
								None => InputMode::first()
									.and_then(|v| v.next())
									.unwrap(),
							};
							app.input_query = String::new();
						}
						/* Copy input query to the clipboard. */
						Key::Ctrl('c') => {
							let query = app.input_query.clone();
							app.set_clipboard_contents(&query);
						}
						/* Paste the clipboard contents. */
						Key::Ctrl('v') => {
							let clipboard_contents = app.get_clipboard_contents();
							app.input_query += &clipboard_contents;
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
								Key::Left => match app.selected_block.previous() {
									Some(v) => v,
									None => Block::last().unwrap(),
								},
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
							/* Set the input mode flag. */
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
				if hide_options {
					app.show_options = false;
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

#[cfg(test)]
mod tests {
	use super::*;
	use clap::ArgMatches;
	use ratatui::backend::TestBackend;
	use std::sync::mpsc::Sender;
	use std::thread;
	use std::time::Duration;
	#[test]
	fn test_tui() -> Result<(), Box<dyn Error>> {
		let args = ArgMatches::default();
		let kernel = Kernel::new(&args);
		let events = Events::new(100, &kernel.logs);
		let tx = events.tx.clone();
		thread::spawn(move || {
			/* Test the general keys. */
			for key in [
				Key::Char('?'),
				Key::Ctrl('t'),
				Key::Ctrl('b'),
				Key::Alt('e'),
				Key::Alt('s'),
				Key::Ctrl('x'),
				Key::Ctrl('x'),
				Key::Ctrl('x'),
				Key::Char('x'),
				Key::Char('n'),
				Key::Char('d'),
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
				Key::Char('m'),
				Key::Down,
				Key::Char('\n'),
			] {
				send_key(&tx, key);
			}
			send_key(&tx, Key::Char('r'));
			/* Test the switch keys. */
			for arrow_key in [Key::Right, Key::Left] {
				for selected_key in [arrow_key; Block::CARDINALITY] {
					send_key(&tx, selected_key);
					for key in [
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
			for key in [
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

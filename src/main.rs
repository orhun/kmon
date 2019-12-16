mod app;
mod event;
mod kernel;
mod util;
use app::{App, Blocks, InputMode};
use enum_unitary::{Bounded, EnumUnitary};
use event::{Event, Events};
use kernel::cmd::ModuleCommand;
use kernel::lkm::KernelModules;
use kernel::log::KernelLogs;
use std::io::stdout;
use termion::event::Key;
use termion::input::MouseTerminal;
use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;
use tui::backend::TermionBackend;
use tui::layout::{Constraint, Direction, Layout};
use tui::Terminal;
use unicode_width::UnicodeWidthStr;
use util::ScrollDirection;

const VERSION: &str = "0.1.0"; /* Version */
const REFRESH_RATE: &str = "250"; /* Default refresh rate of the terminal */

/**
 * Create a terminal instance with using termion as backend.
 *
 * @param  ArgMatches
 * @return Result
 */
fn create_term(args: &clap::ArgMatches) -> Result<(), failure::Error> {
	/* Configure the terminal. */
	let stdout = stdout().into_raw_mode()?;
	let stdout = MouseTerminal::from(stdout);
	let stdout = AlternateScreen::from(stdout);
	let backend = TermionBackend::new(stdout);
	let mut terminal = Terminal::new(backend)?;
	let events = Events::new(
		args.value_of("rate")
			.unwrap_or(REFRESH_RATE)
			.parse::<u64>()
			.unwrap(),
	);
	terminal.hide_cursor()?;
	/* Set required items for the terminal widgets. */
	let mut app = App::new(Blocks::ModuleTable);
	let mut kernel_logs = KernelLogs::new();
	let mut kernel_modules = KernelModules::new(args);
	kernel_modules.scroll_list(ScrollDirection::Top);
	/* Draw terminal and render the widgets. */
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
						app.draw_kernel_version(
							&mut f,
							chunks[1],
							&kernel_logs.version,
						)
					}
					app.draw_kernel_modules(&mut f, chunks[1], &mut kernel_modules);
				}
				app.draw_module_info(&mut f, chunks[1], &mut kernel_modules);
			}
			app.draw_kernel_activities(&mut f, chunks[1], &mut kernel_logs);
		})?;
		/* Set cursor position if the input mode flag is set. */
		if !app.input_mode.is_none() {
			util::set_cursor_pos(
				terminal.backend_mut(),
				2 + app.input_query.width() as u16,
				2,
			)?;
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
						| Key::Ctrl('d') => {
							break;
						}
						/* Refresh. */
						Key::Char('r') | Key::Char('R') | Key::F(5) => {
							app = App::new(Blocks::ModuleTable);
							kernel_logs.scroll_offset = 0;
							kernel_modules = KernelModules::new(args);
							kernel_modules.scroll_list(ScrollDirection::Top);
						}
						/* Show help message. */
						Key::Char('?') => {
							kernel_modules.current_name = String::from("!Help");
							kernel_modules.current_info =
								String::from("(TODO)\nHelp Message")
						}
						/* Scroll the selected block up. */
						Key::Up | Key::Char('k') | Key::Char('K') => match app
							.selected_block
						{
							Blocks::ModuleTable => {
								kernel_modules.scroll_list(ScrollDirection::Up)
							}
							Blocks::ModuleInfo => {
								kernel_modules.scroll_mod_info(ScrollDirection::Up)
							}
							Blocks::Activities => {
								events.tx.send(Event::Input(Key::PageUp)).unwrap();
							}
							_ => {}
						},
						/* Scroll the selected block down. */
						Key::Down | Key::Char('j') | Key::Char('J') => match app
							.selected_block
						{
							Blocks::ModuleTable => {
								kernel_modules.scroll_list(ScrollDirection::Down)
							}
							Blocks::ModuleInfo => {
								kernel_modules.scroll_mod_info(ScrollDirection::Down)
							}
							Blocks::Activities => {
								events.tx.send(Event::Input(Key::PageDown)).unwrap();
							}
							_ => {}
						},
						/* Select the next terminal block. */
						Key::Left
						| Key::Char('h')
						| Key::Char('H')
						| Key::Ctrl('h') => {
							app.selected_block =
								match app.selected_block.prev_variant() {
									Some(v) => v,
									None => Blocks::max_value(),
								}
						}
						/* Select the previous terminal block. */
						Key::Right
						| Key::Char('l')
						| Key::Char('L')
						| Key::Ctrl('l') => {
							app.selected_block =
								match app.selected_block.next_variant() {
									Some(v) => v,
									None => Blocks::min_value(),
								}
						}
						/* Scroll to the top of the module list. */
						Key::Char('t') | Key::Char('T') => {
							app.selected_block = Blocks::ModuleTable;
							kernel_modules.scroll_list(ScrollDirection::Top)
						}
						/* Scroll to the bottom of the module list. */
						Key::Char('b') | Key::Char('B') => {
							app.selected_block = Blocks::ModuleTable;
							kernel_modules.scroll_list(ScrollDirection::Bottom)
						}
						/* Scroll kernel activities up. */
						Key::PageUp => {
							app.selected_block = Blocks::Activities;
							if kernel_logs.scroll_offset > 2 {
								kernel_logs.scroll_offset -= 3;
							}
						}
						/* Scroll kernel activities down. */
						Key::PageDown => {
							app.selected_block = Blocks::Activities;
							if !kernel_logs.output.is_empty() {
								kernel_logs.scroll_offset += 3;
								kernel_logs.scroll_offset %=
									(kernel_logs.output.lines().count() as u16) * 2;
							}
						}
						/* Scroll module information up. */
						Key::Backspace => {
							kernel_modules.scroll_mod_info(ScrollDirection::Up)
						}
						/* Scroll module information down. */
						Key::Char(' ') => {
							kernel_modules.scroll_mod_info(ScrollDirection::Down)
						}
						/* Unload kernel module. */
						Key::Char('u') | Key::Char('U') => {
							kernel_modules
								.set_current_command(ModuleCommand::Unload);
						}
						/* Execute the current command. */
						Key::Char('y') | Key::Char('Y') => {
							if !kernel_modules.command.is_none() {
								match util::exec_cmd(
									"sh",
									&[
										"-c",
										&kernel_modules.get_current_command().cmd,
									],
								) {
									Ok(_) => events
										.tx
										.send(Event::Input(Key::Char('r')))
										.unwrap(),
									Err(e) => {
										kernel_modules.current_info = format!(
											"\nFailed to execute command: '{}'\n\n{}",
											kernel_modules.get_current_command().cmd,
											e
										)
									}
								}
								kernel_modules.command = ModuleCommand::None;
							}
						}
						/* Cancel the execution of current command. */
						Key::Char('n') | Key::Char('N') => {
							if !kernel_modules.command.is_none() {
								app.selected_block = Blocks::ModuleTable;
								kernel_modules.command = ModuleCommand::None;
								if kernel_modules.index != 0 {
									kernel_modules.index -= 1;
									kernel_modules
										.scroll_list(ScrollDirection::Down);
								} else {
									kernel_modules.index += 1;
									kernel_modules.scroll_list(ScrollDirection::Up);
								}
							}
						}
						/* User input mode. */
						Key::Char('\n')
						| Key::Char('s')
						| Key::Char('S')
						| Key::Char('m')
						| Key::Char('M')
						| Key::Char('i')
						| Key::Char('I')
						| Key::Char('/')
						| Key::Home => {
							app.selected_block = Blocks::UserInput;
							match input {
								Key::Char('m')
								| Key::Char('M')
								| Key::Char('i')
								| Key::Char('I') => app.input_mode = InputMode::Load,
								_ => app.input_mode = InputMode::Search,
							}
							if input != Key::Char('\n') {
								app.input_query = String::new();
							}
							util::set_cursor_pos(
								terminal.backend_mut(),
								2 + app.input_query.width() as u16,
								2,
							)?;
							terminal.show_cursor()?;
						}
						_ => {}
					}
				} else {
					/* User input mode. */
					match input {
						/* Quit with ctrl+key combinations. */
						Key::Ctrl('c') | Key::Ctrl('d') => {
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
						/* Exit user input mode. */
						Key::Char('\n')
						| Key::Right
						| Key::Ctrl('l')
						| Key::Left
						| Key::Ctrl('h') => {
							/* Select the next eligible block for action. */
							app.selected_block = match input {
								Key::Left | Key::Ctrl('h') => {
									match app.selected_block.prev_variant() {
										Some(v) => v,
										None => Blocks::max_value(),
									}
								}
								Key::Char('\n') => match app.input_mode {
									InputMode::Load => Blocks::ModuleInfo,
									_ => Blocks::ModuleTable,
								},
								_ => Blocks::ModuleTable,
							};
							/* Show the first modules information if the search mode is set. */
							if app.input_mode == InputMode::Search
								&& kernel_modules.index == 0
							{
								kernel_modules.scroll_list(ScrollDirection::Top);
							/* Load kernel module. */
							} else if app.input_mode == InputMode::Load
								&& !app.input_query.is_empty()
							{
								kernel_modules.current_name = app.input_query;
								kernel_modules
									.set_current_command(ModuleCommand::Load);
								app.input_query = String::new();
							}
							/* Hide terminal cursor and set the input mode flag. */
							terminal.hide_cursor()?;
							app.input_mode = InputMode::None;
						}
						/* Append character to input query. */
						Key::Char(c) => {
							app.input_query.push(c);
							kernel_modules.index = 0;
						}
						/* Delete the last character from input query. */
						Key::Backspace => {
							app.input_query.pop();
							kernel_modules.index = 0;
						}
						_ => {}
					}
				}
			}
			/* Kernel events. */
			Event::Kernel(logs) => {
				kernel_logs.output = logs;
			}
			_ => {}
		}
	}
	Ok(())
}

/**
 * Entry point.
 */
fn main() {
	let matches = util::parse_args(VERSION);
	create_term(&matches).expect("failed to create terminal");
}

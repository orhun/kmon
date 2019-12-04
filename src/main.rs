mod event;
mod kernel;
mod term;
mod util;
use enum_unitary::{Bounded, EnumUnitary};
use event::{Event, Events};
use kernel::log::KernelLogs;
use kernel::module::{KernelModules, ScrollDirection};
use std::io::{self, Write};
use std::time::Duration;
use term::{Blocks, Settings};
use termion::event::Key;
use termion::input::MouseTerminal;
use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;
use tui::backend::TermionBackend;
use tui::layout::{Alignment, Constraint, Direction, Layout};
use tui::style::Modifier;
use tui::widgets::{Block, Borders, Paragraph, Row, Table, Text, Widget};
use tui::Terminal;
use unicode_width::UnicodeWidthStr;
use util::parse_args;

const VERSION: &'static str = "0.1.0"; /* Version */
const REFRESH_RATE: Duration = Duration::from_millis(250); /* Refresh rate of the terminal */
const TABLE_HEADER: [&str; 3] = ["Module", "Size", "Used by"]; /* Header of the kernel modules table */

/**
 * Create a terminal instance with using termion as backend.
 *
 * @param  ArgMatches
 * @return Result
 */
fn create_term(args: &clap::ArgMatches) -> Result<(), failure::Error> {
    /* Configure the terminal. */
    let stdout = io::stdout().into_raw_mode()?;
    let stdout = MouseTerminal::from(stdout);
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let events = Events::new(REFRESH_RATE);
    terminal.hide_cursor()?;
    /* Set required items for the terminal widgets. */
    let mut kernel_logs = KernelLogs::new();
    let mut kernel_modules = KernelModules::new(args);
    let mut settings = Settings::new(Blocks::ModuleTable);
    kernel_modules.scroll_list(ScrollDirection::Top);
    /* Create widgets and draw the terminal. */
    loop {
        terminal.draw(|mut f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(75), Constraint::Percentage(25)].as_ref())
                .split(f.size());
            {
                let chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(60), Constraint::Percentage(40)].as_ref())
                    .split(chunks[0]);
                {
                    let chunks = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([Constraint::Length(3), Constraint::Percentage(100)].as_ref())
                        .split(chunks[0]);
                    /* Search input. */
                    Paragraph::new([Text::raw(&settings.search_query)].iter())
                        .block(
                            Block::default()
                                .title_style(settings.title_style)
                                .border_style(match settings.selected_block {
                                    Blocks::SearchInput => {
                                        if !settings.search_mode {
                                            events.tx.send(Event::Input(Key::Char('\n'))).unwrap();
                                        }
                                        settings.selected_style
                                    }
                                    _ => settings.unselected_style,
                                })
                                .borders(Borders::ALL)
                                .title("Search"),
                        )
                        .render(&mut f, chunks[0]);
                    /* Filter the module list depending on the search query. */
                    let mut kernel_module_list = kernel_modules.default_list.clone();
                    if settings.search_query.len() > 0 {
                        kernel_module_list
                            .retain(|module| module[0].contains(&settings.search_query));
                    }
                    kernel_modules.list = kernel_module_list.clone();
                    /* Set selected and scroll state of the modules. */
                    let modules_scroll_offset = chunks[1]
                        .height
                        .checked_sub(5)
                        .and_then(|height| kernel_modules.index.checked_sub(height as usize))
                        .unwrap_or(0);
                    let modules = kernel_module_list
                        .iter()
                        .skip(modules_scroll_offset)
                        .enumerate()
                        .map(|(i, item)| {
                            if Some(i) == kernel_modules.index.checked_sub(modules_scroll_offset) {
                                Row::StyledData(
                                    item.into_iter(),
                                    settings.selected_style.modifier(Modifier::BOLD),
                                )
                            } else {
                                Row::StyledData(item.into_iter(), settings.selected_style)
                            }
                        });
                    /* Kernel modules table. */
                    Table::new(TABLE_HEADER.into_iter(), modules.into_iter())
                        .block(
                            Block::default()
                                .title_style(settings.title_style)
                                .border_style(settings.block_style(Blocks::ModuleTable))
                                .borders(Borders::ALL)
                                .title(&format!(
                                    "Loaded Kernel Modules ({}/{}) [{}%]",
                                    match kernel_modules.list.len() {
                                        0 => kernel_modules.index,
                                        _ => kernel_modules.index + 1,
                                    },
                                    kernel_modules.list.len(),
                                    ((kernel_modules.index + 1) as f64
                                        / kernel_modules.list.len() as f64
                                        * 100.0) as usize
                                )),
                        )
                        .widths(&[
                            (f64::from(chunks[1].width - 3) * 0.3) as u16,
                            (f64::from(chunks[1].width - 3) * 0.2) as u16,
                            (f64::from(chunks[1].width - 3) * 0.5) as u16,
                        ])
                        .render(&mut f, chunks[1]);
                }
                /* Module information. */
                Paragraph::new([Text::raw(kernel_modules.current_info.to_string())].iter())
                    .block(
                        Block::default()
                            .title_style(settings.title_style)
                            .border_style(settings.block_style(Blocks::ModuleInfo))
                            .borders(Borders::ALL)
                            .title(&format!("Module: {}", kernel_modules.current_name)),
                    )
                    .alignment(Alignment::Left)
                    .wrap(true)
                    .scroll(kernel_modules.info_scroll_offset)
                    .render(&mut f, chunks[1]);
            }
            /* Kernel activities. */
            Paragraph::new([Text::raw(kernel_logs.output.to_string())].iter())
                .block(
                    Block::default()
                        .title_style(settings.title_style)
                        .border_style(settings.block_style(Blocks::Activities))
                        .borders(Borders::ALL)
                        .title("Kernel Activities"),
                )
                .alignment(Alignment::Left)
                .wrap(true)
                .scroll(kernel_logs.scroll_offset)
                .render(&mut f, chunks[1]);
        })?;
        /* Set cursor position and flush stdout. */
        if settings.search_mode {
            write!(
                terminal.backend_mut(),
                "{}",
                termion::cursor::Goto(2 + settings.search_query.width() as u16, 2)
            )?;
            io::stdout().flush().ok();
        }
        /* Handle terminal events. */
        match events.rx.recv()? {
            /* Key input events. */
            Event::Input(input) => {
                if !settings.search_mode {
                    /* Default input mode. */
                    match input {
                        /* Quit. */
                        Key::Char('q') | Key::Char('Q') | Key::Ctrl('c') | Key::Ctrl('d') => {
                            break;
                        }
                        /* Refresh. */
                        Key::Char('r') | Key::Char('R') => {
                            kernel_logs.scroll_offset = 0;
                            kernel_modules = KernelModules::new(args);
                            kernel_modules.scroll_list(ScrollDirection::Top);
                        }
                        /* Scroll the selected block up. */
                        Key::Up | Key::Char('k') | Key::Char('K') => {
                            match settings.selected_block {
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
                            }
                        }
                        /* Scroll the selected block down. */
                        Key::Down | Key::Char('j') | Key::Char('J') => {
                            match settings.selected_block {
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
                            }
                        }
                        /* Scroll to the top of the module list. */
                        Key::Char('t') | Key::Char('T') => {
                            kernel_modules.scroll_list(ScrollDirection::Top)
                        }
                        /* Scroll to the bottom of the module list. */
                        Key::Char('b') | Key::Char('B') => {
                            kernel_modules.scroll_list(ScrollDirection::Bottom)
                        }
                        /* Select the next terminal block. */
                        Key::Left | Key::Char('h') | Key::Char('H') => {
                            settings.selected_block = match settings.selected_block.prev_variant() {
                                Some(v) => v,
                                None => Blocks::max_value(),
                            }
                        }
                        /* Select the previous terminal block. */
                        Key::Right | Key::Char('l') | Key::Char('L') => {
                            settings.selected_block = match settings.selected_block.next_variant() {
                                Some(v) => v,
                                None => Blocks::min_value(),
                            }
                        }
                        /* Scroll kernel activities up. */
                        Key::PageUp => {
                            settings.selected_block = Blocks::Activities;
                            if kernel_logs.scroll_offset > 2 {
                                kernel_logs.scroll_offset -= 3;
                            }
                        }
                        /* Scroll kernel activities down. */
                        Key::PageDown => {
                            settings.selected_block = Blocks::Activities;
                            if kernel_logs.output.len() > 0 {
                                kernel_logs.scroll_offset += 3;
                                kernel_logs.scroll_offset %=
                                    (kernel_logs.output.lines().count() as u16) * 2;
                            }
                        }
                        /* Search in modules. */
                        Key::Char('\n') | Key::Char('s') | Key::Char('/') | Key::Home => {
                            settings.selected_block = Blocks::SearchInput;
                            if input != Key::Char('\n') {
                                settings.search_query = String::new();
                            }
                            write!(
                                terminal.backend_mut(),
                                "{}",
                                termion::cursor::Goto(2 + settings.search_query.width() as u16, 2)
                            )?;
                            terminal.show_cursor()?;
                            settings.search_mode = true;
                        }
                        _ => {}
                    }
                } else {
                    /* Search mode. */
                    match input {
                        /* Quit with ctrl+key combinations. */
                        Key::Ctrl('c') | Key::Ctrl('d') => {
                            break;
                        }
                        /* Exit search mode. */
                        Key::Char('\n') | Key::Right | Key::Left => {
                            if input == Key::Left {
                                settings.selected_block =
                                    match settings.selected_block.prev_variant() {
                                        Some(v) => v,
                                        None => Blocks::max_value(),
                                    }
                            } else {
                                settings.selected_block = Blocks::ModuleTable;
                            }
                            terminal.hide_cursor()?;
                            settings.search_mode = false;
                        }
                        /* Append character to search query. */
                        Key::Char(c) => {
                            settings.search_query.push(c);
                            kernel_modules.index = 0;
                        }
                        /* Delete last character from search query. */
                        Key::Backspace => {
                            settings.search_query.pop();
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
    let matches = parse_args(VERSION);
    create_term(&matches).expect("failed to create terminal");
}

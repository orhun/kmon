mod event;
mod kernel;
mod util;
use event::{Event, Events};
use kernel::log::KernelLogs;
use kernel::module::{KernelModules, ScrollDirection};
use std::io::{self, Write};
use std::time::Duration;
use termion::event::Key;
use termion::input::MouseTerminal;
use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;
use tui::backend::TermionBackend;
use tui::layout::{Alignment, Constraint, Direction, Layout};
use tui::style::{Color, Modifier, Style};
use tui::widgets::{Block, Borders, Paragraph, Row, Table, Text, Widget};
use tui::Terminal;
use unicode_width::UnicodeWidthStr;
use util::parse_args;

const VERSION: &'static str = "0.1.0"; /* Version */
const REFRESH_RATE: Duration = Duration::from_millis(250); /* Refresh rate of the terminal */
const TABLE_HEADER: [&str; 3] = ["Module", "Size", "Used by"]; /* Header of the kernel modules table */
enum Blocks {
    SearchInput,
    ModuleTable,
    ModuleInfo,
    Activities,
}

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
    kernel_modules.scroll_list(ScrollDirection::Top);
    let mut search_query = String::new();
    let mut search_mode = false;
    let mut selected_block = Blocks::ModuleTable;
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
                    Paragraph::new([Text::raw(&search_query)].iter())
                        .block(
                            Block::default()
                                .title_style(Style::default().modifier(Modifier::BOLD))
                                .border_style(match selected_block {
                                    Blocks::SearchInput => Style::default().fg(Color::Cyan),
                                    _ => Style::default(),
                                })
                                .borders(Borders::ALL)
                                .title("Search"),
                        )
                        .render(&mut f, chunks[0]);
                    /* Filter the module list depending on the search query. */
                    let mut kernel_module_list = kernel_modules.default_list.clone();
                    if search_query.len() > 0 {
                        kernel_module_list.retain(|module| module[0].contains(&search_query));
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
                                    Style::default().fg(Color::White).modifier(Modifier::BOLD),
                                )
                            } else {
                                Row::StyledData(item.into_iter(), Style::default().fg(Color::White))
                            }
                        });
                    /* Kernel modules table. */
                    Table::new(TABLE_HEADER.into_iter(), modules.into_iter())
                        .block(
                            Block::default()
                                .title_style(Style::default().modifier(Modifier::BOLD))
                                .border_style(match selected_block {
                                    Blocks::ModuleTable => Style::default().fg(Color::Cyan),
                                    _ => Style::default(),
                                })
                                .borders(Borders::ALL)
                                .title(&format!(
                                    "Loaded Kernel Modules ({}/{}) [{}%]",
                                    kernel_modules.index + 1,
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
                            .title_style(Style::default().modifier(Modifier::BOLD))
                            .border_style(match selected_block {
                                    Blocks::ModuleInfo => Style::default().fg(Color::Cyan),
                                    _ => Style::default(),
                                })
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
                        .title_style(Style::default().modifier(Modifier::BOLD))
                        .border_style(match selected_block {
                                    Blocks::Activities => Style::default().fg(Color::Cyan),
                                    _ => Style::default(),
                                })
                        .borders(Borders::ALL)
                        .title("Kernel Activities"),
                )
                .alignment(Alignment::Left)
                .wrap(true)
                .scroll(kernel_logs.scroll_offset)
                .render(&mut f, chunks[1]);
        })?;
        /* Set cursor position and flush stdout. */
        if search_mode {
            write!(
                terminal.backend_mut(),
                "{}",
                termion::cursor::Goto(2 + search_query.width() as u16, 2)
            )?;
            io::stdout().flush().ok();
        }
        /* Handle terminal events. */
        match events.rx.recv()? {
            /* Key input events. */
            Event::Input(input) => {
                if !search_mode {
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
                        /* Scroll through the kernel modules and show information. */
                        Key::Up | Key::Char('k') | Key::Char('K') => {
                            kernel_modules.scroll_list(ScrollDirection::Up)
                        }
                        Key::Down | Key::Char('j') | Key::Char('J') => {
                            kernel_modules.scroll_list(ScrollDirection::Down)
                        }
                        Key::Char('t') | Key::Char('T') => {
                            kernel_modules.scroll_list(ScrollDirection::Top)
                        }
                        Key::Char('b') | Key::Char('B') => {
                            kernel_modules.scroll_list(ScrollDirection::Bottom)
                        }
                        /* Scroll the module information up. */
                        Key::Left | Key::Char('h') | Key::Char('H') => {
                            kernel_modules.scroll_mod_info(ScrollDirection::Up)
                        }
                        /* Scroll the module information down. */
                        Key::Right | Key::Char('l') | Key::Char('L') => {
                            kernel_modules.scroll_mod_info(ScrollDirection::Down)
                        }
                        /* Scroll kernel activities up. */
                        Key::PageUp => {
                            if kernel_logs.scroll_offset > 2 {
                                kernel_logs.scroll_offset -= 3;
                            }
                        }
                        /* Scroll kernel activities down. */
                        Key::PageDown => {
                            if kernel_logs.output.len() > 0 {
                                kernel_logs.scroll_offset += 3;
                                kernel_logs.scroll_offset %=
                                    (kernel_logs.output.lines().count() as u16) * 2;
                            }
                        }
                        /* Search in modules. */
                        Key::Char('\n') | Key::Char('s') | Key::Char('/') | Key::Home => {
                            if input != Key::Char('\n') {
                                search_query = String::new();
                            }
                            write!(
                                terminal.backend_mut(),
                                "{}",
                                termion::cursor::Goto(2 + search_query.width() as u16, 2)
                            )?;
                            terminal.show_cursor()?;
                            search_mode = true;
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
                        Key::Char('\n') => {
                            terminal.hide_cursor()?;
                            search_mode = false;
                        }
                        /* Append character to search query. */
                        Key::Char(c) => {
                            search_query.push(c);
                            kernel_modules.index = 0;
                        }
                        /* Delete last character from search query. */
                        Key::Backspace => {
                            search_query.pop();
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
            Event::Tick => {}
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

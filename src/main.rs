use bytesize::ByteSize;
use clap::App;
use clap::Arg;
use clap::SubCommand;
use std::io::{self, Write};
use std::process::Command;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use termion::event::Key;
use termion::input::MouseTerminal;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;
use tui::backend::TermionBackend;
use tui::layout::{Alignment, Constraint, Direction, Layout};
use tui::style::{Color, Modifier, Style};
use tui::widgets::{Block, Borders, Paragraph, Row, Table, Text, Widget};
use tui::Terminal;

const VERSION: &'static str = "0.1.0";                                /* Version */
const REFRESH_RATE: std::time::Duration = Duration::from_millis(250); /* Refresh rate of the terminal */

enum Event<I> { /* Terminal events enumerator */
    Input(I),
    Kernel(Vec<tui::widgets::Text<'static>>),
    Tick,
}
#[allow(dead_code)]
struct Events { /* Terminal events struct */
    rx: mpsc::Receiver<Event<Key>>,
    input_handler: thread::JoinHandle<()>,
    kernel_handler: thread::JoinHandle<()>,
    tick_handler: thread::JoinHandle<()>,
}

/**
 * Execute a operating system command and return its output.
 *
 * @param  cmd
 * @param  cmd_args
 * @return Result
 */
fn exec_cmd(cmd: &str, cmd_args: &[&str]) -> Result<String, String> {
    let output = Command::new(cmd)
        .args(cmd_args)
        .output()
        .expect("failed to execute command");
    /* Write error output to stderr stream. */
    io::stderr().write_all(&output.stderr).unwrap();
    if output.status.success() {
        Ok((String::from_utf8(output.stdout).expect("not UTF-8"))
            .trim_end()
            .to_string())
    } else {
        Err(format!("{} {}", cmd, cmd_args.join(" ")))
    }
}

fn get_kernel_modules(args: clap::ArgMatches) -> ([&str; 3], Vec<Vec<String>>) {
    let module_headers = ["Module", "Size", "Used by"];
    let mut kernel_modules: Vec<Vec<String>> = Vec::new();
    let mut module_read_cmd = String::from("cat /proc/modules");
    if let Some(matches) = args.subcommand_matches("sort") {
        if matches.is_present("size") {
            module_read_cmd += " | sort -n -r -t ' ' -k2";
        } else {
            module_read_cmd += " | sort -t ' ' -k1";
        }
    }
    let modules_content =
        exec_cmd("sh", &["-c", &module_read_cmd]).expect("failed to read /proc/modules");
    for line in modules_content.lines() {
        let columns = line.split_whitespace().collect::<Vec<&str>>();
        let mut module_name = columns[0].to_string();
        if columns.len() >= 7 {
            module_name = format!("{} {}", module_name, columns[6]);
        }
        let mut used_modules = format!("{} {}", columns[2], columns[3]);
        if used_modules.chars().last().unwrap() == ',' {
            used_modules.pop();
        }
        let module_size = ByteSize::b(columns[1].to_string().parse().unwrap()).to_string();
        kernel_modules.push(vec![module_name, module_size, used_modules]);
    }
    (module_headers, kernel_modules)
}

fn get_next_index(direction_up: bool, current_index: usize, elements_count: usize) -> usize {
    let mut next_index = current_index;
    if direction_up {
        if next_index > 0 {
            next_index -= 1;
        } else {
            next_index = elements_count - 1;
        }
    } else {
        next_index += 1;
        if next_index > elements_count - 1 {
            next_index = 0;
        }
    }
    next_index
}

/**
 * Return terminal events after setting handlers.
 *
 * @return Events
 */
fn get_events() -> Events {
    /* Create a new asynchronous channel. */
    let (tx, rx) = mpsc::channel();
    /* Handle inputs using stdin stream and sender of the channel. */
    let input_handler = {
        let tx = tx.clone();
        thread::spawn(move || {
            let stdin = io::stdin();
            for evt in stdin.keys() {
                match evt {
                    Ok(key) => {
                        tx.send(Event::Input(key)).unwrap();
                    }
                    Err(_) => {}
                }
            }
        })
    };
    /* Handle kernel logs with getting 'dmesg' output. */
    let kernel_handler = {
        let tx = tx.clone();
        thread::spawn(move || {
            let tx = tx.clone();
            loop {
                let dmesg_output = exec_cmd("dmesg", &["--kernel", "--human", "--color=never"])
                    .expect("failed to retrieve dmesg output");
                tx.send(Event::Kernel(
                    dmesg_output
                        .lines()
                        .rev()
                        .map(|x| Text::raw(format!("{}\n", x)))
                        .collect(),
                ))
                .unwrap();
                thread::sleep(REFRESH_RATE * 20);
            }
        })
    };
    /* Create a loop for handling events. */
    let tick_handler = {
        let tx = tx.clone();
        thread::spawn(move || {
            let tx = tx.clone();
            loop {
                tx.send(Event::Tick).unwrap();
                thread::sleep(REFRESH_RATE);
            }
        })
    };
    /* Return events. */
    Events {
        rx,
        input_handler,
        kernel_handler,
        tick_handler,
    }
}

/**
 * Create a terminal instance with using termion as backend.
 *
 * @param  ArgMatches
 * @return Result
 */
fn create_term(args: clap::ArgMatches) -> Result<(), failure::Error> {
    /* Configure the terminal. */
    let stdout = io::stdout().into_raw_mode()?;
    let stdout = MouseTerminal::from(stdout);
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let events = get_events();
    terminal.hide_cursor()?;
    let mut kernel_logs: Vec<tui::widgets::Text> = Vec::new();
    let (module_headers, kernel_modules) = get_kernel_modules(args);
    let mut selected_index: usize = 0;
    let mut module_info = String::from("-");
    let mut module_info_scroll_offset: u16 = 0;
    let mut selected_module_name = String::from("-");
    /* Set widgets and draw the terminal. */
    loop {
        terminal.draw(|mut f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints(
                    [
                        Constraint::Percentage(15),
                        Constraint::Percentage(60),
                        Constraint::Percentage(25),
                    ]
                    .as_ref(),
                )
                .split(f.size());
            {
                let chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
                    .split(chunks[0]);
                Block::default()
                    .title("Row 1 Block 1")
                    .borders(Borders::ALL)
                    .render(&mut f, chunks[0]);
                Block::default()
                    .title("Row 1 Block 2")
                    .borders(Borders::ALL)
                    .render(&mut f, chunks[1]);
            }
            {
                let chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(60), Constraint::Percentage(40)].as_ref())
                    .split(chunks[1]);
                let modules_scroll_offset = chunks[0]
                    .height
                    .checked_sub(5)
                    .and_then(|height| selected_index.checked_sub(height as usize))
                    .unwrap_or(0);
                let modules =
                    kernel_modules
                        .iter()
                        .skip(modules_scroll_offset)
                        .enumerate()
                        .map(|(i, item)| {
                            if Some(i) == selected_index.checked_sub(modules_scroll_offset) {
                                Row::StyledData(
                                    item.into_iter(),
                                    Style::default().fg(Color::White).modifier(Modifier::BOLD),
                                )
                            } else {
                                Row::StyledData(item.into_iter(), Style::default().fg(Color::White))
                            }
                        });
                Table::new(module_headers.into_iter(), modules.into_iter())
                    .block(
                        Block::default()
                            .title_style(Style::default().modifier(Modifier::BOLD))
                            .borders(Borders::ALL)
                            .title(&format!(
                                "Loaded Kernel Modules ({}/{})",
                                selected_index + 1,
                                kernel_modules.len()
                            )),
                    )
                    .widths(&[
                        (f64::from(chunks[0].width - 3) * 0.3) as u16,
                        (f64::from(chunks[0].width - 3) * 0.2) as u16,
                        (f64::from(chunks[0].width - 3) * 0.5) as u16,
                    ])
                    .render(&mut f, chunks[0]);
                Paragraph::new([Text::raw(module_info.to_string())].iter())
                    .block(
                        Block::default()
                            .title_style(Style::default().modifier(Modifier::BOLD))
                            .borders(Borders::ALL)
                            .title(&format!("Module: {}", selected_module_name)),
                    )
                    .alignment(Alignment::Left)
                    .wrap(true)
                    .scroll(module_info_scroll_offset)
                    .render(&mut f, chunks[1]);
            }
            {
                let chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(100)].as_ref())
                    .split(chunks[2]);
                Paragraph::new(kernel_logs.iter())
                    .block(
                        Block::default()
                            .title_style(Style::default().modifier(Modifier::BOLD))
                            .borders(Borders::ALL)
                            .title("Kernel Activities"),
                    )
                    .alignment(Alignment::Left)
                    .wrap(true)
                    .render(&mut f, chunks[0]);
            }
        })?;
        /* Handle terminal events. */
        match events.rx.recv()? {
            Event::Input(input) => match input {
                Key::Char('q') | Key::Char('Q') | Key::Ctrl('c') | Key::Ctrl('d') => {
                    break;
                }
                Key::Down | Key::Up => {
                    selected_index =
                        get_next_index(input == Key::Up, selected_index, kernel_modules.len());
                    selected_module_name = kernel_modules[selected_index][0]
                            .split(" (")
                            .collect::<Vec<&str>>()[0].to_string();
                    module_info = exec_cmd(
                        "modinfo",
                        &[&selected_module_name],
                    )
                    .unwrap();
                }
                Key::Right => {
                    module_info_scroll_offset += 1;
                }
                Key::Left => {
                    if module_info_scroll_offset > 0 {
                        module_info_scroll_offset -= 1;
                    }
                }
                _ => {}
            },
            Event::Kernel(logs) => {
                kernel_logs = logs;
            }
            Event::Tick => {}
        }
    }
    Ok(())
}

/**
 * Parse command line arguments using 'clap'.
 *
 * @return ArgMatches
 */
fn parse_args() -> clap::ArgMatches<'static> {
    App::new("kmon")
        .version(VERSION)
        .subcommand(
            SubCommand::with_name("sort")
                .about("Sort kernel modules")
                .arg(
                    Arg::with_name("size")
                        .short("s")
                        .long("size")
                        .help("Sort modules by their sizes"),
                )
                .arg(
                    Arg::with_name("name")
                        .short("n")
                        .long("name")
                        .help("Sort modules by their names"),
                ),
        )
        .get_matches()
}

/**
 * Entry point.
 */
fn main() {
    let matches = parse_args();
    create_term(matches).expect("failed to create terminal");
}

/**
 * Unit test module.
 */
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_exec_cmd() {
        assert_eq!("test", exec_cmd("printf", &["test"]).unwrap());
        assert_eq!(
            "true",
            exec_cmd("sh", &["-c", "test 10 -eq 10 && echo 'true'"]).unwrap()
        );
    }
    #[test]
    fn test_get_kernel_modules() {
        let matches = App::new("test").get_matches();
        assert_ne!(0, get_kernel_modules(matches).len());
    }
    #[test]
    fn test_get_events() -> Result<(), failure::Error> {
        let events = get_events();
        match events.rx.recv()? {
            Event::Input(_) => Ok(()),
            Event::Tick => Ok(()),
            Event::Kernel(logs) => {
                if logs.len() > 0 {
                    Ok(())
                } else {
                    Err(failure::err_msg("failed to retrieve kernel logs"))
                }
            }
        }
    }
}

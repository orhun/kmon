use bytesize::ByteSize;
use clap::App;
use clap::Arg;
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
#[allow(dead_code)]
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

fn get_kernel_modules(sort_modules: bool) -> Vec<Vec<String>> {
    let mut module_read_cmd = String::from("cat /proc/modules");
    if sort_modules {
        module_read_cmd += " | sort -n -r -t ' ' -k2";
    }
    let modules_content = exec_cmd("sh", &["-c", &module_read_cmd])
        .expect("failed to read /proc/modules");
    let mut kernel_modules: Vec<Vec<String>> = Vec::new();
    for line in modules_content.lines() {
        let columns = line.split_whitespace().collect::<Vec<&str>>();
        kernel_modules.push(vec![columns[0].to_string(),
            ByteSize::b(columns[1].to_string()
                .parse().unwrap()).to_string(),
            columns[3].to_string()]);
    }
    kernel_modules
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
                let dmesg_output =
                    exec_cmd("dmesg", &["--kernel", "--human",
                        "--color=never"])
                        .expect("failed to retrieve dmesg output");
                tx.send(Event::Kernel(
                    dmesg_output
                        .lines()
                        .rev()
                        .map(|x| Text::raw(format!("{}\n", x)))
                        .collect(),
                )).unwrap();
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
    let header = ["Header1", "Header2", "Header3"];
    let kernel_modules = get_kernel_modules(args.is_present("sort"));
    let mut selected_index: usize = 0;
    /* Set widgets and draw the terminal. */
    loop {
        terminal.draw(|mut f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints(
                    [
                        Constraint::Percentage(25),
                        Constraint::Percentage(50),
                        Constraint::Percentage(25),
                    ]
                    .as_ref(),
                )
                .split(f.size());
            {
                let chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(50),
                        Constraint::Percentage(50)].as_ref())
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
                    .constraints([Constraint::Percentage(50),
                        Constraint::Percentage(50)].as_ref())
                    .split(chunks[1]);
                let scroll_offset = chunks[0]
                    .height
                    .checked_sub(5)
                    .and_then(|height| selected_index.checked_sub(height as usize))
                    .unwrap_or(0);
                let modules = kernel_modules.iter().skip(scroll_offset)
                    .enumerate().map(|(i, item)| {
                    if Some(i) == selected_index.checked_sub(scroll_offset) {
                        Row::StyledData(item.into_iter(),
                            Style::default().fg(Color::White).modifier(Modifier::BOLD))
                    } else {
                        Row::StyledData(item.into_iter(), Style::default().fg(Color::White))
                    }
                });
                Table::new(header.into_iter(), modules.into_iter())
                    .block(Block::default()
                        .title_style(Style::default().modifier(Modifier::BOLD))
                        .borders(Borders::ALL).title("Row 2 Block 1"))
                    .widths(&[
                        (f64::from(chunks[0].width - 3) * 0.3) as u16,
                        (f64::from(chunks[0].width - 3) * 0.5) as u16,
                        (f64::from(chunks[0].width - 3) * 0.1) as u16])
                    .render(&mut f, chunks[0]);
                Block::default()
                    .title("Row 2 Block 2")
                    .borders(Borders::ALL)
                    .render(&mut f, chunks[1]);
            }
            {
                let chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(100)].as_ref())
                    .split(chunks[2]);
                Paragraph::new(kernel_logs.iter())
                    .block(Block::default()
                        .title_style(Style::default().modifier(Modifier::BOLD))
                        .borders(Borders::ALL).title("Kernel Activities"))
                    .alignment(Alignment::Left)
                    .wrap(true)
                    .render(&mut f, chunks[0]);
            }
        })?;
        /* Handle terminal events. */
        match events.rx.recv()? {
            Event::Input(input) => match input {
                Key::Char('q') | Key::Char('Q') |
                Key::Ctrl('c') | Key::Ctrl('d') => {
                    break;
                },
                Key::Down => {
                    selected_index += 1;
                    if selected_index > kernel_modules.len() - 1 {
                        selected_index = 0;
                    }
                },
                Key::Up => {
                    if selected_index > 0 {
                        selected_index -= 1;
                    } else {
                        selected_index = kernel_modules.len() - 1;
                    }
                },
                _ => {}
            }
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
 */
fn parse_args() -> clap::ArgMatches<'static>  {
    App::new("kmon").version(VERSION)
        .arg(Arg::with_name("sort")
                               .short("s")
                               .long("sort")
                               .help("Sort kernel modules by size"))
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

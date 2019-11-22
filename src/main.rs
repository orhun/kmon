use clap::App;
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
use tui::style::{Modifier, Style};
use tui::widgets::{Block, Borders, Paragraph, SelectableList, Text, Widget};
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

/**
 * Parse command line arguments using 'clap'.
 */
fn parse_args() {
    let _matches = App::new("kmon").version(VERSION).get_matches();
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
                        "--color=never"]).unwrap();
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
 * @return Result
 */
fn create_term() -> Result<(), failure::Error> {
    /* Configure the terminal. */
    let stdout = io::stdout().into_raw_mode()?;
    let stdout = MouseTerminal::from(stdout);
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let events = get_events();
    terminal.hide_cursor()?;
    let mut kernel_logs: Vec<tui::widgets::Text> = Vec::new();
    let selected_index: usize = 0;
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
                Block::default()
                    .title("Row 2 Block 4")
                    .borders(Borders::ALL)
                    .render(&mut f, chunks[1]);
                {
                    let chunks = Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints(
                            [
                                Constraint::Percentage(30),
                                Constraint::Percentage(20),
                                Constraint::Percentage(50),
                            ]
                            .as_ref(),
                        )
                        .split(chunks[0]);
                    let block =
                        Block::default().title_style(
                            Style::default().modifier(Modifier::BOLD));
                    SelectableList::default()
                        .block(block.clone().title(" Row 2 Block 1"))
                        .items(&vec![" Item1", " Item2"])
                        .select(Some(selected_index))
                        .highlight_symbol(">>")
                        .render(&mut f, chunks[0]);
                    Block::default()
                        .title("Row 2 Block 2")
                        .render(&mut f, chunks[1]);
                    Block::default()
                        .title("Row 2 Block 3")
                        .render(&mut f, chunks[2]);
                }
            }
            {
                let chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(100)].as_ref())
                    .split(chunks[2]);
                let block = Block::default()
                    .title_style(Style::default().modifier(Modifier::BOLD))
                    .borders(Borders::ALL);
                Paragraph::new(kernel_logs.iter())
                    .block(block.clone().title("Kernel Activities"))
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
                }
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
 * Entry point.
 */
fn main() {
    parse_args();
    create_term().expect("failed to create terminal");
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

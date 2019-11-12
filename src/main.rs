use std::io::{self, Write};
use std::sync::mpsc;
use std::thread;
use std::process::Command;
use std::time::Duration;
use clap::{App};
use tui::Terminal;
use tui::backend::TermionBackend;
use tui::widgets::{Widget, Block, Borders};
use tui::layout::{Layout, Constraint, Direction};
use termion::event::Key;
use termion::raw::IntoRawMode;
use termion::input::TermRead;
use termion::input::MouseTerminal;
use termion::screen::AlternateScreen;

const VERSION: &'static str = "0.1.0"; /* Version */
const EXIT_KEY:termion::event::Key = Key::Char('q');
const TICK_RATE:std::time::Duration = Duration::from_millis(250);

enum Event<I> {
    Input(I),
    Tick,
}
struct Events {
    rx: mpsc::Receiver<Event<Key>>,
    input_handle: thread::JoinHandle<()>,
    tick_handle: thread::JoinHandle<()>,
}


/**
 * Execute a operating system command and return its output.
 *
 * @param  cmd
 * @param  cmd_args
 * @return result
 */
#[allow(dead_code)]
fn exec_cmd(cmd: &str, cmd_args: &[&str]) -> Result<String, String> {
    let output = Command::new(cmd).args(cmd_args)
        .output().expect("failed to execute command");
    /* Write error output to stderr stream. */
    io::stderr().write_all(&output.stderr).unwrap();
    if output.status.success() {
        Ok((String::from_utf8(output.stdout).expect("not UTF-8"))
            .trim_end().to_string())
    } else {
        Err(format!("{} {}", cmd, cmd_args.join(" ")))
    }
}

/**
 * Parse command line arguments using 'clap'.
 */
fn parse_args() {
    let _matches = App::new("kmon")
        .version(VERSION).get_matches();
}

fn get_term_events() -> Events {
    let (tx, rx) = mpsc::channel();
    let input_handle = {
        let tx = tx.clone();
        thread::spawn(move || {
            let stdin = io::stdin();
            for evt in stdin.keys() {
                match evt {
                    Ok(key) => {
                        if let Err(_) = tx.send(Event::Input(key)) {
                            return;
                        }
                        if key == EXIT_KEY {
                            return;
                        }
                    }
                    Err(_) => {}
                }
            }
        })
    };
    let tick_handle = {
        let tx = tx.clone();
        thread::spawn(move || {
            let tx = tx.clone();
            loop {
                tx.send(Event::Tick).unwrap();
                thread::sleep(TICK_RATE);
            }
        })
    };
    Events {
        rx,
        input_handle,
        tick_handle,
    }
}

/**
 * Create a terminal instance using termion as backend.
 *
 * @return result
 */
fn create_term() -> Result<(), io::Error> {
    let stdout = io::stdout().into_raw_mode()?;
    let stdout = MouseTerminal::from(stdout);
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let events = get_term_events();
    terminal.hide_cursor()?;
    loop {
        terminal.draw(|mut f| {
            let size = f.size();
                Block::default().borders(Borders::ALL).render(&mut f, size);
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                            Constraint::Percentage(25),
                            Constraint::Percentage(50),
                            Constraint::Percentage(25)
                        ].as_ref()).split(f.size());
                {
                    let chunks = Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints([
                            Constraint::Percentage(50),
                            Constraint::Percentage(50)
                        ].as_ref()).split(chunks[0]);
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
                        .constraints([
                            Constraint::Percentage(50),
                            Constraint::Percentage(50)
                            ].as_ref()).split(chunks[1]);
                    Block::default()
                        .title("Row 2 Block 1")
                        .borders(Borders::ALL)
                        .render(&mut f, chunks[0]);
                    Block::default()
                        .title("Row 2 Block 2")
                        .borders(Borders::ALL)
                        .render(&mut f, chunks[1]);
                }
                {
                    let chunks = Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints([
                            Constraint::Percentage(20),
                            Constraint::Percentage(80)
                            ].as_ref()).split(chunks[2]);
                    Block::default()
                        .title("Row 3 Block 1")
                        .borders(Borders::ALL)
                        .render(&mut f, chunks[0]);
                    Block::default()
                        .title("Row 3 Block 2")
                        .borders(Borders::ALL)
                        .render(&mut f, chunks[1]);
                }
        })?;
        break; // TODO: Add event handler.
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
        assert_eq!("true", exec_cmd("sh", &["-c",
            "test 10 -eq 10 && echo 'true'"]).unwrap());
    }
}

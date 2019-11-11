use std::io::{self, Write};
use std::process::Command;
use clap::{App};
use tui::Terminal;
use tui::backend::TermionBackend;
use tui::widgets::{Widget, Block, Borders};
use tui::layout::{Layout, Constraint, Direction};
use termion::raw::IntoRawMode;


const VERSION: &'static str = "0.1.0"; /* Version */

/**
 * Execute a operating system command and return its output.
 *
 * @param  cmd
 * @param  cmd_args
 * @return result
 */
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

/**
 * Create a terminal instance using termion as backend.
 *
 * @return result
 */
fn create_term() -> Result<(), io::Error> {
    let stdout = io::stdout().into_raw_mode()?;
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.draw(|mut f| {
        let size = f.size();
            Block::default().borders(Borders::ALL).render(&mut f, size);
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
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
                    .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
                    .split(chunks[1]);
                Block::default()
                    .title("Row 2 Block 1")
                    .borders(Borders::ALL)
                    .render(&mut f, chunks[0]);
                Block::default()
                    .title("Row 2 Block 2")
                    .borders(Borders::ALL)
                    .render(&mut f, chunks[1]);
            }
    })
}

/**
 * Entry point.
 */
fn main() {
    parse_args();
    println!("{}", exec_cmd("sh", &["-c", "echo 'x'"]).unwrap());
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

use std::io;
use std::sync::mpsc;
use std::thread;
use termion::input::TermRead;
use termion::event::Key;
use crate::kernel::log::KernelLogs;

/* Terminal events enumerator */
pub enum Event<I> {
    Input(I),
    Kernel(String),
    Tick,
}
/* Terminal events struct and implementation. */
#[allow(dead_code)]
pub struct Events {
    pub rx: mpsc::Receiver<Event<Key>>,
    input_handler: thread::JoinHandle<()>,
    kernel_handler: thread::JoinHandle<()>,
    tick_handler: thread::JoinHandle<()>,
}
impl Events {
    /**
     * Create a new events instance.
     *
     * @param  refresh_rate
     * @return Events
     */
    pub fn new(refresh_rate: std::time::Duration) -> Self {
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
        /* Handle kernel logs using 'dmesg' output. */
        let kernel_handler = {
            let tx = tx.clone();
            thread::spawn(move || {
                let tx = tx.clone();
                let mut kernel_logs = KernelLogs::new();
                loop {
                    if kernel_logs.update() {
                        tx.send(Event::Kernel(kernel_logs.output.to_string()))
                            .unwrap();
                    }
                    thread::sleep(refresh_rate * 10);
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
                    thread::sleep(refresh_rate);
                }
            })
        };
        /* Return events. */
        Self {
            rx,
            input_handler,
            kernel_handler,
            tick_handler,
        }
    }
}

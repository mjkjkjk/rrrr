use std::fs::OpenOptions;
use std::io::Write;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;

pub struct Logger {
    sender: Sender<String>,
}

impl Logger {
    pub fn new(log_file: String) -> Self {
        let (sender, receiver) = channel();

        thread::spawn(move || {
            log_worker(receiver, log_file);
        });

        Logger { sender }
    }

    pub fn log(&self, command: String) {
        if let Err(e) = self.sender.send(command) {
            eprintln!("Failed to send log message: {}", e);
        }
    }
}

fn log_worker(receiver: Receiver<String>, log_file: String) {
    let mut file = match OpenOptions::new().create(true).append(true).open(&log_file) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Failed to open log file {}: {}", log_file, e);
            return;
        }
    };

    while let Ok(command) = receiver.recv() {
        let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
        if let Err(e) = writeln!(file, "[{}] {}", timestamp, command) {
            eprintln!("Failed to write to log file: {}", e);
        }
    }
}

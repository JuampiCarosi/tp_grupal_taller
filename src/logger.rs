use std::sync::mpsc;
use std::fs::{OpenOptions, File};
use std::io::prelude::*;
use std::path::Path;
pub struct Logger {
    rx: mpsc::Receiver<String>,
    log_file_path: String,
}

impl Logger {
    pub fn new(rx: mpsc::Receiver<String>) -> Logger {
        println!("Logger started");

        let file_path = "../log.txt";
        let path = Path::new(file_path);

        if !path.exists() {
            File::create(path).unwrap();
        }
        Logger { rx, log_file_path: file_path.to_string() }
    }

    pub fn start_logging(&self) {
        let mut data_file = OpenOptions::new()
            .append(true)
            .open(&self.log_file_path)
            .expect("cannot open file");

        while let Ok(msg) = &self.rx.recv() {
            data_file.write(format!("{}\n", msg).as_bytes()).expect("write failed");
            println!("Escribi {}", msg);
        }
    }
}

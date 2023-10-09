use std::io::prelude::*;
use std::{
    fs::{File, OpenOptions},
    path::Path,
    sync::mpsc,
    thread::{self, JoinHandle},
};

use chrono::Local;

pub enum Log {
    Message(String),
    End,
}

pub struct Logger {
    tx: mpsc::Sender<Log>,
    handle: Option<JoinHandle<()>>,
}

impl Logger {
    pub fn new() -> Result<Logger, String> {
        for _ in 0..8 {
            let (tx, rx) = mpsc::channel();
            let logger_thread = thread::Builder::new().name("Logger".to_string());

            let handle_result = logger_thread.spawn(move || {
                let file_path = "./log.txt";
                let path = Path::new(file_path);

                if !path.exists() {
                    File::create(path).unwrap();
                }

                let mut data_file = OpenOptions::new()
                    .append(true)
                    .open(file_path)
                    .expect("cannot open file");

                // println!("Logger started");
                println!("{:?}", data_file);
                loop {
                    match rx.recv() {
                        Ok(Log::Message(msg)) => {
                            let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");

                            data_file
                                .write(format!("{} | {}\n", timestamp, msg).as_bytes())
                                .expect("write failed");
                            println!("Escribi {}", msg);
                        }
                        Ok(Log::End) => break,
                        Err(_) => break,
                    }
                }
            });

            if let Ok(handle) = handle_result {
                return Ok(Logger {
                    tx,
                    handle: Some(handle),
                });
            }
        }

        println!("No se pudo crear el logger");
        Err("No se pudo crear el logger".to_string())
    }

    pub fn log(&self, msg: String) {
        let log = Log::Message(msg.clone());
        if self.tx.send(log).is_err() {
            println!("No se pudo escribir {}", msg);
        };
    }
}

impl Drop for Logger {
    fn drop(&mut self) {
        if self.tx.send(Log::End).is_err() {
            return;
        };

        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
        println!("Logger dropped");
    }
}

use std::{
    sync::{mpsc, Arc},
    thread::{self, JoinHandle},
};

pub enum Log {
    Message(String),
    End,
}

pub struct Logger {
    tx: mpsc::Sender<Log>,
    handle: Option<JoinHandle<()>>,
}

impl Logger {
    pub fn new() -> Arc<Logger> {
        let (tx, rx) = mpsc::channel();

        let handle = thread::spawn(move || {
            println!("Logger started");
            loop {
                match rx.recv() {
                    Ok(Log::Message(msg)) => println!("{}", msg),
                    Ok(Log::End) => break,
                    Err(_) => break,
                }
            }
        });

        Arc::new(Logger {
            tx,
            handle: Some(handle),
        })
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

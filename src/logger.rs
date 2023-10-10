use std::io::prelude::*;
use std::{
    env,
    fs::{File, OpenOptions},
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
    pub fn new() -> Result<Logger, Box<dyn std::error::Error>> {
        let (tx, rx) = mpsc::channel();
        let file_path: String;

        match get_file_path() { 
            Some(file) => file_path = file,
            None => file_path = "./log.txt".to_string(),
        }
        
        let mut data_file = OpenOptions::new()
            .append(true)
            .open(file_path)?;


        let logger_thread = thread::Builder::new().name("Logger".to_string());
        let handle_result = logger_thread.spawn(move || {
            
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
        } else {
            // Aca podria ir un error que creamos nosotros
            Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                "No se pudo crear el logger",
            )))        
        }
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


fn get_file_path() -> Option<String> {
    let file_path;
    match env::current_dir() {
        Ok(current_dir) => {
            file_path = current_dir.as_path().join("log.txt");
            if !file_path.clone().exists() {
                File::create(file_path.clone()).unwrap();
            }
            Some(String::from(file_path.to_str()?))
        }
        Err(err) => {
            eprintln!("Error al obtener el directorio de trabajo actual: {}", err);
            None
        }
    }
}
// use std::sync::mpsc;
// use std::fs::{OpenOptions, File};
// use std::io::{prelude::*, Error};
// use std::env;
// pub struct Logger {
//     rx: mpsc::Receiver<String>,
//     log_file_path: String,
// }

// impl Logger {
  
//     pub fn new(rx: mpsc::Receiver<String>) -> Logger {
//         println!("Logger started");
//         let file_path: String;
//         match get_file_path() { 
//             Some(file) => file_path = file,
//             None => file_path = "./log.txt".to_string(),
//         }
//         Logger { rx, log_file_path: file_path }
//     }
    
//     pub fn start_logging(&self) -> Result<(), Error> {
//         let mut data_file = OpenOptions::new()
//             .append(true)
//             .open(&self.log_file_path)?;

//         while let Ok(msg) = &self.rx.recv() {
//             data_file.write(format!("{}\n", msg).as_bytes())?;
//             // println!("Escribi {}", msg);
//         }
//         Ok(())
//     }
// }


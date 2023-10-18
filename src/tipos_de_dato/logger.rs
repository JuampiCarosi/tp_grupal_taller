use chrono::Local;
use std::io::prelude::*;
use std::path::PathBuf;
use std::{
    env,
    fs::{File, OpenOptions},
    sync::{mpsc, mpsc::Sender},
    thread::{self, JoinHandle},
};

use crate::io::crear_archivo;

//que use el modulo io el logger
//hay que mover a otro archivo
/// Represents a log message or the end of the logger thread.
pub enum Log {
    /// A log message.
    Message(String),
    /// The end of the logger thread.
    End,
}

/// A logger that writes messages to a file.
pub struct Logger {
    tx: Sender<Log>,
    handle: Option<JoinHandle<()>>,
}

impl Logger {
    /// Creates a new logger that writes messages to a file.
    ///
    /// If the current working directory cannot be obtained or the log file cannot be opened,
    /// the logger will write messages to a file named "log.txt" in the current directory.
    pub fn new() -> Result<Logger, String> {
        let (tx, rx) = mpsc::channel();

        let archivo_log = Self::obtener_archivo_log()?;

        let handle = Self::crear_logger_thread(rx, archivo_log)?;

        Ok(Logger {
            tx,
            handle: Some(handle),
        })
    }

    /// Writes a log message to the file.
    pub fn log(&self, msg: String) {
        let log = Log::Message(msg.clone());
        if self.tx.send(log).is_err() {
            println!("No se pudo escribir {}", msg);
        };
    }

    fn crear_logger_thread(
        rx: mpsc::Receiver<Log>,
        mut archivo_log: File,
    ) -> Result<JoinHandle<()>, String> {
        let logger_thread = thread::Builder::new().name("Logger".to_string());

        logger_thread
            .spawn(move || loop {
                match rx.recv() {
                    Ok(Log::Message(msg)) => {
                        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");

                        let _ = escribir_mensaje_en_archivo_log(&mut archivo_log, timestamp, msg);
                    }
                    Ok(Log::End) => break,
                    Err(_) => break,
                }
            })
            .map_err(|err| format!("ERROR: No se pudo crear el logger.\n{}", err))
    }

    fn obtener_archivo_log() -> Result<File, String> {
        let dir_archivo_log = Self::obtener_dir_archivo_log()?;
        OpenOptions::new()
            .append(true)
            .open(dir_archivo_log)
            .map_err(|err| format!("{}", err))
    }

    fn obtener_dir_archivo_log() -> Result<PathBuf, String> {
        let dir_actual = Self::obtener_directorio_actual()?;

        let dir_archivo_log = dir_actual.as_path().join("log.txt");

        crear_archivo(dir_archivo_log.clone())?;

        Ok(dir_archivo_log)
    }

    fn obtener_directorio_actual() -> Result<PathBuf, String> {
        let dir_actual = env::current_dir().map_err(|err| format!("{}", err))?;
        Ok(dir_actual)
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
        println!("Logger cerrado");
    }
}

fn escribir_mensaje_en_archivo_log(
    data_archivo: &mut File,
    timestamp: chrono::format::DelayedFormat<chrono::format::StrftimeItems<'_>>,
    msg: String,
) -> Result<(), String> {
    data_archivo
        .write_all(format!("{} | {}\n", timestamp, msg).as_bytes())
        .map_err(|err| format!("{}", err))?;
    Ok(())
}

#[cfg(test)]
mod test {
    use super::Logger;
    use std::{env, fs, sync::Arc, thread};
    extern crate serial_test;

    use serial_test::serial;

    #[test]
    #[serial]
    fn test01_al_iniciar_si_archivo_log_no_esta_creado_se_crea() {
        eliminar_archivo_log();

        Logger::new().unwrap();

        assert!(obtener_dir_archivo_log().exists());
    }

    #[test]
    #[serial]
    fn test02_se_escribe_correctamente_los_mensajes_archivo_log() {
        eliminar_archivo_log();
        let logger = Logger::new().unwrap();

        let msg_test_01 = "sipiropo fapatapalapa".to_string();
        let msg_test_02 = "juapuanipi peperezpez".to_string();

        logger.log(msg_test_01.clone());
        logger.log(msg_test_02.clone());
        drop(logger);

        assert_el_archivo_log_contiene(vec![msg_test_01, msg_test_02])
    }

    #[test]
    #[serial]
    fn test03_si_se_crea_un_logger_no_se_pierden_los_mensajes_anterior() {
        eliminar_archivo_log();

        let msg_test_01 = "sipiropo fapatapalapa".to_string();
        Logger::new().unwrap().log(msg_test_01.clone());

        let msg_test_02 = "juapuanipi peperezpez".to_string();
        Logger::new().unwrap().log(msg_test_02.clone());

        assert_el_archivo_log_contiene(vec![msg_test_01, msg_test_02]);
    }

    #[test]
    #[serial]
    fn test04_el_logger_puede_escribir_mensajes_de_varios_threads() {
        eliminar_archivo_log();

        let logger = Arc::new(Logger::new().unwrap());

        let msg_test_01 = Arc::new("Thread 1 saluda".to_string());
        let msg_test_01_copia_para_thread = msg_test_01.clone();
        let msg_test_02 = Arc::new("Thread 2 saluda".to_string());
        let msg_test_02_copia_para_thread = msg_test_02.clone();
        let msg_test_03 = Arc::new("Thread 3 saluda".to_string());
        let msg_test_03_copia_para_thread = msg_test_03.clone();

        let logger1 = logger.clone();
        let handle_1 = thread::spawn(move || {
            logger1.log(msg_test_01_copia_para_thread.to_string());
        });

        let logger2 = logger.clone();
        let handle_2 = thread::spawn(move || {
            logger2.log(msg_test_02_copia_para_thread.to_string());
        });

        let logger3 = logger.clone();
        let handle_3 = thread::spawn(move || {
            logger3.log(msg_test_03_copia_para_thread.to_string());
        });

        handle_1.join().unwrap();
        handle_2.join().unwrap();
        handle_3.join().unwrap();
        drop(logger);

        assert_el_archivo_log_contiene(vec![
            msg_test_01.to_string(),
            msg_test_02.to_string(),
            msg_test_03.to_string(),
        ]);
    }

    fn assert_el_archivo_log_contiene(contenidos: Vec<String>) {
        let contenido_archvo_log = fs::read_to_string(obtener_dir_archivo_log()).unwrap();

        for contenido in contenidos {
            assert!(contenido_archvo_log.contains(&contenido));
        }
    }

    fn eliminar_archivo_log() {
        let dir_archivo_log = obtener_dir_archivo_log();
        if dir_archivo_log.exists() {
            fs::remove_file(dir_archivo_log.clone()).unwrap();
        }
    }

    fn obtener_dir_archivo_log() -> std::path::PathBuf {
        env::current_dir().unwrap().as_path().join("log.txt")
    }
}

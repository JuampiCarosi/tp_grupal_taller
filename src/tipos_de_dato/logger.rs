use chrono::Local;
use std::io::prelude::*;
use std::path::PathBuf;
use std::{
    env,
    fs::{File, OpenOptions},
    sync::{mpsc, mpsc::Sender},
    thread::{self, JoinHandle},
};

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

    fn obtener_dir_archivo_log() -> Result<String, String> {
        let dir_actual = Self::obtener_directorio_actual()?;

        let dir_archivo_log = dir_actual.as_path().join("log.txt");

        if !dir_archivo_log.exists() {
            Self::crear_archivo_log(&dir_archivo_log)?;
        }

        dir_archivo_log
            .to_str()
            .ok_or_else(|| String::from("Error al convertir el path a String"))
            .map(String::from)
    }

    fn crear_archivo_log(dir_archivo_log: &PathBuf) -> Result<(), String> {
        File::create(dir_archivo_log.clone()).map_err(|err| format!("{}", err))?;
        Ok(())
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
    use std::{env, fs};

    #[test]

    fn test01_al_iniciar_si_archivo_log_no_esta_creado_se_crea() {
        eliminar_archivo_log();

        Logger::new().unwrap();

        assert!(obtener_dir_archivo_log().exists());
    }

    //no entiendo porque no pasa este test, debugeando lo pasa
    #[test]
    fn test02_se_escribe_correctamente_un_mensajes_archivo_log() {
        eliminar_archivo_log();
        let logger = Logger::new().unwrap();

        let archivo_log = obtener_dir_archivo_log();
        assert!(archivo_log.exists());

        let msg_test = "sipiropo fapatapalapa".to_string();

        logger.log(msg_test.clone());

        let contenido_archvo_log = fs::read_to_string(obtener_dir_archivo_log()).unwrap();

        println!("{}", contenido_archvo_log);
        let assert = contenido_archvo_log.contains(&msg_test);
        assert!(assert);
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

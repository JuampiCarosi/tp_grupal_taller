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
    pub fn new(ubicacion_archivo: PathBuf) -> Result<Logger, String> {
        let (tx, rx) = mpsc::channel();

        let ubicacion_archivo_completa = Self::obtener_archivo_log(ubicacion_archivo)?;

        let handle = Self::crear_logger_thread(rx, ubicacion_archivo_completa)?;

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

    fn obtener_archivo_log(ubicacion_archivo: PathBuf) -> Result<File, String> {
        let dir_archivo_log = Self::obtener_dir_archivo_log(ubicacion_archivo)?;
        OpenOptions::new()
            .append(true)
            .open(dir_archivo_log)
            .map_err(|err| format!("{}", err))
    }

    fn obtener_dir_archivo_log(ubicacion_archivo: PathBuf) -> Result<PathBuf, String> {
        if ubicacion_archivo.is_absolute() {
            crear_archivo(&ubicacion_archivo)?;
            return Ok(ubicacion_archivo);
        }

        let dir_actual = Self::obtener_directorio_actual()?;

        let dir_archivo_log = dir_actual.as_path().join(ubicacion_archivo);

        crear_archivo(&dir_archivo_log)?;

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
    use std::{env, fs, path::PathBuf, sync::Arc, thread};
    extern crate serial_test;

    #[test]
    fn test01_al_iniciar_si_archivo_log_no_esta_creado_se_crea() {
        let ubicacion_archivo = PathBuf::from("test_dir/test01.txt");
        Logger::new(ubicacion_archivo.clone()).unwrap();

        assert!(obtener_dir_archivo_log(ubicacion_archivo.clone()).exists());
        eliminar_archivo_log(ubicacion_archivo);
    }

    #[test]
    fn test02_se_escribe_correctamente_los_mensajes_archivo_log() {
        let ubicacion_archivo = PathBuf::from("test_dir/test02.txt");
        let logger = Logger::new(ubicacion_archivo.clone()).unwrap();

        let msg_test_01 = "sipiropo fapatapalapa".to_string();
        let msg_test_02 = "juapuanipi peperezpez".to_string();

        logger.log(msg_test_01.clone());
        logger.log(msg_test_02.clone());
        drop(logger);

        assert_el_archivo_log_contiene(ubicacion_archivo.clone(), vec![msg_test_01, msg_test_02]);
        eliminar_archivo_log(ubicacion_archivo);
    }

    #[test]
    fn test03_si_se_crea_un_logger_no_se_pierden_los_mensajes_anterior() {
        let msg_test_01 = "sipiropo fapatapalapa".to_string();
        let ubicacion_archivo = PathBuf::from("test_dir/test03.txt");
        Logger::new(ubicacion_archivo.clone())
            .unwrap()
            .log(msg_test_01.clone());

        let msg_test_02 = "juapuanipi peperezpez".to_string();
        Logger::new(ubicacion_archivo.clone())
            .unwrap()
            .log(msg_test_02.clone());

        assert_el_archivo_log_contiene(ubicacion_archivo.clone(), vec![msg_test_01, msg_test_02]);
        eliminar_archivo_log(ubicacion_archivo);
    }

    #[test]
    fn test04_el_logger_puede_escribir_mensajes_de_varios_threads() {
        let ubicacion_archivo = PathBuf::from("test_dir/test04.txt");
        let logger = Arc::new(Logger::new(ubicacion_archivo.clone()).unwrap());
        let msg_test_01 = "Thread 1 saluda".to_string();
        let msg_test_02 = "Thread 2 saluda".to_string();
        let msg_test_03 = "Thread 3 saluda".to_string();

        let handle_1 = crear_thread_que_mande_mensaje_al_loger(&logger, msg_test_01.clone());
        let handle_2 = crear_thread_que_mande_mensaje_al_loger(&logger, msg_test_02.clone());
        let handle_3 = crear_thread_que_mande_mensaje_al_loger(&logger, msg_test_03.clone());

        handle_1.join().unwrap();
        handle_2.join().unwrap();
        handle_3.join().unwrap();
        drop(logger);

        assert_el_archivo_log_contiene(
            ubicacion_archivo.clone(),
            vec![msg_test_01, msg_test_02, msg_test_03],
        );
        eliminar_archivo_log(ubicacion_archivo);
    }

    fn crear_thread_que_mande_mensaje_al_loger(
        logger: &Arc<Logger>,
        msg: String,
    ) -> thread::JoinHandle<()> {
        let logger1 = logger.clone();

        thread::spawn(move || {
            logger1.log(msg);
        })
    }

    fn assert_el_archivo_log_contiene(ubicacion_archivo: PathBuf, contenidos: Vec<String>) {
        let contenido_archvo_log =
            fs::read_to_string(obtener_dir_archivo_log(ubicacion_archivo)).unwrap();

        for contenido in contenidos {
            assert!(contenido_archvo_log.contains(&contenido));
        }
    }

    fn eliminar_archivo_log(ubicacion_archivo: PathBuf) {
        let dir_archivo_log = obtener_dir_archivo_log(ubicacion_archivo);
        if dir_archivo_log.exists() {
            fs::remove_file(dir_archivo_log.clone()).unwrap();
        }
    }

    fn obtener_dir_archivo_log(ubicacion_archivo: PathBuf) -> std::path::PathBuf {
        env::current_dir()
            .unwrap()
            .as_path()
            .join(ubicacion_archivo)
    }
}

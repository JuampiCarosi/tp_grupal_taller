use std::{
    path::PathBuf,
    sync::{mpsc::channel, Arc, Mutex},
};

use gir::{
    servidor::{
        gir_server::ServidorGir, http_server::ServidorHttp,
        rutas::mensaje_servidor::MensajeServidor,
    },
    tipos_de_dato::logger::Logger,
};

fn main() -> Result<(), String> {
    let logger = Arc::new(Logger::new(PathBuf::from("server_logger.txt"))?);

    let (tx, rx) = channel::<MensajeServidor>();
    let threads = Arc::new(Mutex::new(Vec::new()));

    let mut servidor_http = ServidorHttp::new(logger.clone(), threads.clone(), tx)?;
    servidor_http.iniciar_servidor()?;

    let mut servidor_gir = ServidorGir::new(logger, threads.clone())?;
    servidor_gir.iniciar_servidor()?;

    rx.recv().unwrap();

    let mut threads = threads.lock().unwrap();

    for thread in threads.drain(..) {
        thread.join().unwrap()?;
    }

    Ok(())
}

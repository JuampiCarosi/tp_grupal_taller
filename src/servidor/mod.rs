use std::{
    path::PathBuf,
    sync::{
        mpsc::{channel, Receiver, Sender},
        Arc, Mutex,
    },
};

use gir::{
    servidor::{
        gir_server::ServidorGir, http_server::ServidorHttp,
        rutas::mensaje_servidor::MensajeServidor,
    },
    tipos_de_dato::logger::Logger,
};

fn correr_servidor(
    intentos_restantes_gir: u8,
    intentos_restantes_http: u8,
    logger: Arc<Logger>,
    channel: (Sender<MensajeServidor>, Receiver<MensajeServidor>),
    threads: Arc<Mutex<Vec<std::thread::JoinHandle<Result<(), String>>>>>,
) -> Result<(), String> {
    if intentos_restantes_gir == 0 || intentos_restantes_http == 0 {
        return Err("No se pudo reiniciar el servidor".to_owned());
    }

    let (tx, rx) = channel;

    let mut servidor_http = ServidorHttp::new(logger.clone(), threads.clone(), tx.clone())?;
    servidor_http.iniciar_servidor()?;

    let mut servidor_gir = ServidorGir::new(logger.clone(), threads.clone())?;
    servidor_gir.iniciar_servidor()?;

    let error = rx.recv().unwrap();

    drop(servidor_gir);
    drop(servidor_http);

    match error {
        MensajeServidor::GirErrorFatal => correr_servidor(
            intentos_restantes_gir - 1,
            intentos_restantes_http,
            logger,
            (tx, rx),
            threads,
        ),
        MensajeServidor::HttpErrorFatal => correr_servidor(
            intentos_restantes_gir,
            intentos_restantes_http - 1,
            logger,
            (tx, rx),
            threads,
        ),
    }?;

    Ok(())
}

fn main() -> Result<(), String> {
    let logger = Arc::new(Logger::new(PathBuf::from("server_logger.txt"))?);

    let channel = channel::<MensajeServidor>();
    let threads = Arc::new(Mutex::new(Vec::new()));

    correr_servidor(3, 3, logger, channel, threads.clone())?;

    let mut threads = threads.lock().unwrap();

    for thread in threads.drain(..) {
        thread.join().unwrap()?;
    }

    Ok(())
}

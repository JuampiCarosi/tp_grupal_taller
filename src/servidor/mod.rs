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

const INTENTOS_REINICIO: u8 = 3;

fn correr_servidor(
    logger: Arc<Logger>,
    channel: (Sender<MensajeServidor>, Receiver<MensajeServidor>),
    threads: Arc<Mutex<Vec<std::thread::JoinHandle<Result<(), String>>>>>,
) -> Result<(), String> {
    let (tx, rx) = channel;

    let mut intentos_restantes_gir = INTENTOS_REINICIO;
    let mut intentos_restantes_http = INTENTOS_REINICIO;

    let mut servidor_http = ServidorHttp::new(logger.clone(), threads.clone(), tx.clone())?;
    servidor_http.iniciar_servidor()?;

    let mut servidor_gir = ServidorGir::new(logger.clone(), threads.clone(), tx.clone())?;
    servidor_gir.iniciar_servidor()?;

    while let Ok(error_servidor) = rx.recv() {
        match error_servidor {
            MensajeServidor::GirErrorFatal => {
                intentos_restantes_gir -= 1;
                servidor_gir.reiniciar_servidor()?;
            }
            MensajeServidor::HttpErrorFatal => {
                intentos_restantes_http -= 1;
                servidor_http.reiniciar_servidor()?;
            }
        };

        if intentos_restantes_gir == 0 || intentos_restantes_http == 0 {
            return Err("No se pudo reiniciar el servidor".to_owned());
        }
    }

    Ok(())
}

fn main() -> Result<(), String> {
    let logger = Arc::new(Logger::new(PathBuf::from("server_logger.txt"))?);

    let channel = channel::<MensajeServidor>();
    let threads = Arc::new(Mutex::new(Vec::new()));

    correr_servidor(logger.clone(), channel, threads.clone())?;

    if let Ok(mut threads) = threads.lock() {
        for handle in threads.drain(..) {
            let _ = handle.join();
        }
    } else {
        logger.log("Error al obtener el lock de threads desde main");
    }

    Ok(())
}

use std::{env::args, path::PathBuf, sync::Arc, thread};

use gir::{
    servidor::{gir_server::ServidorGir, http_server::ServidorHttp},
    tipos_de_dato::logger::Logger,
    utils::gir_config,
};

static SERVER_ARGS: usize = 2;

fn servidor_gir(logger: Arc<Logger>) -> Result<(), String> {
    let argv = args().collect::<Vec<String>>();
    if argv.len() != SERVER_ARGS {
        println!("Cantidad de argumentos inválido");
        let app_name = &argv[0];
        println!("Usage:\n{:?} <puerto>", app_name);
        return Err("Cantidad de argumentos inválido".to_string());
    }

    let address = "127.0.0.1:".to_owned() + &argv[1];
    let mut servidor = ServidorGir::new(&address, logger).map_err(|e| e.to_string())?;
    servidor.iniciar_servidor()?;

    Ok(())
}

fn servidor_http(logger: Arc<Logger>) -> Result<(), String> {
    let puerto = gir_config::conseguir_puerto_http()
        .ok_or("No se pudo conseguir el puerto http, revise el archivo config")?;

    let address = "127.0.0.1:".to_owned() + &puerto;
    let mut servidor = ServidorHttp::new(&address, logger).map_err(|e| e.to_string())?;
    servidor.iniciar_servidor()?;

    Ok(())
}

fn main() -> Result<(), String> {
    let logger = Arc::new(Logger::new(PathBuf::from("server_logger.txt"))?);

    let logger_clone = logger.clone();
    let gir_handle = thread::spawn(move || servidor_gir(logger_clone));

    let logger_clone = logger.clone();
    let http_handle = thread::spawn(move || servidor_http(logger_clone));

    gir_handle.join().expect("Gir thread panicked")?;
    http_handle.join().expect("HTTP thread panicked")?;

    Ok(())
}

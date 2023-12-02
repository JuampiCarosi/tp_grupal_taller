use gir::{servidor::server::Servidor, tipos_de_dato::logger::Logger};
use std::{env::args, path::PathBuf, sync::Arc};
static SERVER_ARGS: usize = 2;
fn main() -> Result<(), String> {
    let argv = args().collect::<Vec<String>>();
    if argv.len() != SERVER_ARGS {
        println!("Cantidad de argumentos inválido");
        let app_name = &argv[0];
        println!("Usage:\n{:?} <puerto>", app_name);
        return Err("Cantidad de argumentos inválido".to_string());
    }

    let logger = Arc::new(Logger::new(PathBuf::from("server_logger.txt"))?);
    let address = "127.0.0.1:".to_owned() + &argv[1];
    let mut servidor = Servidor::new(&address, logger).map_err(|e| e.to_string())?;
    servidor.iniciar_servidor()?;

    // let mut sv = Servidor::new(&address).unwrap();
    // sv.server_run().unwrap();
    Ok(())
}

use std::{path::PathBuf, rc::Rc};

use gir::tipos_de_dato::{comando::Comando, logger::Logger};

use gir::io;

const DIR_ARCHIVO_CONFIG: &str = "~/.girconfig";

//extrae la ubiacion del archivo log seteada en el archivo config. En caso de error
// devuelve una direccion default = .log
fn obtener_dir_archivo_log(ubicacion_config: PathBuf) -> String {
    let mut dir_archivo_log = ".log".to_string();

    let contenido_config = match io::leer_a_string(ubicacion_config) {
        Ok(contenido_config) => contenido_config,
        Err(_) => return dir_archivo_log,
    };

    for linea_config in contenido_config.lines() {
        if linea_config.trim().starts_with("log") {
            if let Some(dir_archivo_log_config) = linea_config.split('=').nth(1) {
                dir_archivo_log = dir_archivo_log_config.trim().to_string();
                break;
            }
        }
    }

    dir_archivo_log
}

fn main() -> Result<(), String> {
    let args = std::env::args().collect::<Vec<String>>();
    let logger = Rc::new(Logger::new(PathBuf::from(obtener_dir_archivo_log(
        DIR_ARCHIVO_CONFIG.into(),
    )))?);

    let mut comando = match Comando::new(args, logger.clone()) {
        Ok(comando) => comando,
        Err(err) => {
            logger.log(err);
            return Ok(());
        }
    };

    match comando.ejecutar() {
        Ok(mensaje) => {
            logger.log(mensaje.clone());
            println!("{}", mensaje);
        }
        Err(mensaje) => {
            logger.log(mensaje);
        }
    };

    Ok(())
}

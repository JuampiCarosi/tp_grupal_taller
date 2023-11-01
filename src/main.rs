use std::{path::PathBuf, rc::Rc};

use gir::tipos_de_dato::{comando::Comando, logger::Logger};

use gir::io;

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
    let logger = Rc::new(Logger::new(PathBuf::from("log.txt"))?);

    if args.len() < 2 {
        println!("ERROR: No se ingreso ningun comando");
        return Ok(());
    }

    if args[1] == "gui" {
        gir::gui::ejecutar(logger.clone());
        return Ok(());
    }

    let mut comando = match Comando::new(args, logger.clone()) {
        Ok(comando) => comando,
        Err(err) => {
            logger.log(err);
            return Ok(());
        }
    };

    match comando.ejecutar() {
        Ok(mensaje) => {
            println!("{}", mensaje);
            logger.log(mensaje);
        }
        Err(mensaje) => {
            println!("ERROR: {}", mensaje);
            logger.log(mensaje);
        }
    };

    Ok(())
}

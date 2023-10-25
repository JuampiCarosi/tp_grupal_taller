use std::{path::PathBuf, rc::Rc};

use gir::tipos_de_dato::{comando::Comando, logger::Logger};

fn main() -> Result<(), String> {
    let args = std::env::args().collect::<Vec<String>>();
    let logger = Rc::new(Logger::new(PathBuf::from("log.txt"))?);

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

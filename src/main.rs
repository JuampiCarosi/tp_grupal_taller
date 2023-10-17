use std::rc::Rc;

use gir::tipos_de_dato::{comando::Comando, logger::Logger};

fn main() -> Result<(), String> {
    let args = std::env::args().collect::<Vec<String>>();
    let logger = Rc::new(Logger::new()?);

    let comando = match Comando::new(args, logger.clone()) {
        Ok(comando) => comando,
        Err(err) => {
            logger.log(err);
            return Ok(());
        }
    };

    if let Err(mensaje) = comando.ejecutar() {
        logger.log(mensaje.clone());
        return Err(mensaje.clone());
    }

Ok(())
}

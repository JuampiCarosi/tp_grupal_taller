use std::rc::Rc;

use gir::tipos_de_dato::{comando::Comando, logger::Logger};

fn main() -> Result<(), String> {
    let args = std::env::args().collect::<Vec<String>>();
    let logger = Rc::new(Logger::new().unwrap());

    let comando = Comando::new(args, logger.clone())?;
    comando.ejecutar()?;
    Ok(())
}

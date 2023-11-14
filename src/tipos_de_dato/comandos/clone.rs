use crate::tipos_de_dato::logger::Logger;

// use gir::tipos_de_dato::{comando::Comando, logger::Logger};

use std::sync::Arc;

use super::init::Init;
use super::pull::Pull;
// use gir::tipos_de_dato::{comando::Comando, logger::Logger};

//-------- ATENCION ----------
// Si hay una ref que no apunta a nada porque esta vacia, rompe al hacer el split de refs.

pub struct Clone {
    logger: Arc<Logger>,
}

impl Clone {
    pub fn from(logger: Arc<Logger>) -> Result<Self, String> {
        Ok(Clone { logger })
    }

    pub fn ejecutar(&mut self) -> Result<String, String> {
        Init::from(Vec::new(), self.logger.clone())?.ejecutar()?;
        Pull::from(self.logger.clone())?.ejecutar()?;

        let mensaje = "Clone ejecutado con exito".to_string();
        self.logger.log(mensaje.clone());
        Ok(mensaje)
    }
}

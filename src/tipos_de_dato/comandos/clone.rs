use crate::tipos_de_dato::comunicacion::Comunicacion;
use crate::tipos_de_dato::logger::Logger;

use std::net::TcpStream;
// use gir::tipos_de_dato::{comando::Comando, logger::Logger};

use std::sync::Arc;

use super::init::Init;
use super::pull::Pull;
// use gir::tipos_de_dato::{comando::Comando, logger::Logger};

//-------- ATENCION ----------
// Si hay una ref que no apunta a nada porque esta vacia, rompe al hacer el split de refs.

pub struct Clone {
    remoto: String,
    comunicacion: Arc<Comunicacion<TcpStream>>,
    logger: Arc<Logger>,
}

impl Clone {
    pub fn from(logger: Arc<Logger>, comunicacion: Arc<Comunicacion<TcpStream>>) -> Self {
        let remoto = "origin".to_string();
        Clone {
            logger,
            comunicacion,
            remoto,
        }
    }

    pub fn ejecutar(&mut self) -> Result<String, String> {
        Init::from(Vec::new(), self.logger.clone())?.ejecutar()?;
        Pull::from(self.logger.clone(), self.comunicacion.clone())?.ejecutar()?;

        let mensaje = format!("Clone ejecutado con exito");
        self.logger.log(mensaje.clone());
        Ok(mensaje)
    }
}

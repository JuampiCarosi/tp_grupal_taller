use std::rc::Rc;

use crate::tipos_de_dato::{
    comandos::cat_file::CatFile, logger::Logger, visualizaciones::Visualizaciones,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Blob {
    pub nombre: String,
    pub hash: String,
}

impl Blob {
    pub fn obtener_hash(&self) -> String {
        self.hash.clone()
    }
    pub fn obtener_tamanio(&self) -> usize {
        let logger = Rc::new(Logger::new().unwrap());

        let cat_file = CatFile {
            logger,
            visualizacion: Visualizaciones::Tamanio,
            objeto: self.hash.clone(),
        };

        let tamanio_string = cat_file.ejecutar().unwrap();

        tamanio_string.parse::<usize>().unwrap()
    }
}

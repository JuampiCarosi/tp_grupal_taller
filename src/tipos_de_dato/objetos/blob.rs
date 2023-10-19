use std::rc::Rc;

use crate::tipos_de_dato::{
    comando::Comando, comandos::cat_file::CatFile, logger::Logger, visualizaciones::Visualizaciones,
};

#[derive(Debug, Clone)]
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

        let cat_file = Comando::CatFile(CatFile {
            logger,
            visualizacion: Visualizaciones::Tamanio,
            objeto: self.nombre.clone(),
        });

        let tamanio_string = cat_file.ejecutar().unwrap();

        tamanio_string.parse::<usize>().unwrap()
    }
}

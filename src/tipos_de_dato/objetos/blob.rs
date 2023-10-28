use crate::{
    tipos_de_dato::{
        comandos::{cat_file::conseguir_tamanio, hash_object::HashObject},
        logger::Logger,
    },
    utilidades_path_buf::obtener_nombre,
};
use std::{fmt::Display, path::PathBuf, rc::Rc};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Blob {
    pub hash: String,
    pub ubicacion: PathBuf,
    pub nombre: String,
}

impl Blob {
    pub fn obtener_hash(&self) -> String {
        self.hash.clone()
    }
    pub fn obtener_tamanio(&self) -> Result<usize, String> {
        let tamanio_blob = match conseguir_tamanio(self.hash.clone()) {
            Ok(tamanio) => tamanio,
            Err(_) => return Err("No se pudo obtener el tamanio del blob".to_string()),
        };
        match tamanio_blob.parse::<usize>() {
            Ok(tamanio) => Ok(tamanio),
            Err(_) => Err("No se pudo parsear el tamanio del blob".to_string()),
        }
    }

    pub fn from_directorio(directorio: PathBuf) -> Result<Blob, String> {
        let logger = Rc::new(Logger::new(PathBuf::from("tmp/objeto"))?);
        let hash = HashObject {
            logger,
            escribir: false,
            ubicacion_archivo: directorio.clone(),
        }
        .ejecutar()?;

        let nombre = obtener_nombre(&directorio)?;

        Ok(Blob {
            nombre,
            hash,
            ubicacion: directorio,
        })
    }
}

impl Display for Blob {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string = format!("100644 {} {}\n", self.hash, self.ubicacion.display());
        write!(f, "{}", string)
    }
}

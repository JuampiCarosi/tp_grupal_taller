use crate::{
    tipos_de_dato::{
        comandos::{cat_file::conseguir_tamanio, hash_object::HashObject},
        logger::Logger,
    },
    utils::compresion::descomprimir_objeto,
    utils::path_buf::obtener_nombre,
};
use std::{fmt::Display, path::PathBuf, sync::Arc};

#[derive(Clone, Debug)]
pub struct Blob {
    pub hash: String,
    pub ubicacion: PathBuf,
    pub nombre: String,
    pub logger: Arc<Logger>,
}

impl PartialEq for Blob {
    fn eq(&self, other: &Self) -> bool {
        self.hash == other.hash
    }
}

impl Eq for Blob {}

impl Blob {
    pub fn obtener_hash(&self) -> String {
        self.hash.clone()
    }
    pub fn obtener_tamanio(&self) -> Result<usize, String> {
        let contenido_blob = descomprimir_objeto(self.hash.clone())?;
        let tamanio_blob = conseguir_tamanio(contenido_blob)?;
        match tamanio_blob.parse::<usize>() {
            Ok(tamanio) => Ok(tamanio),
            Err(_) => Err("No se pudo parsear el tamanio del blob".to_string()),
        }
    }

    pub fn from_directorio(directorio: PathBuf, _logger: Arc<Logger>) -> Result<Blob, String> {
        let logger = Arc::new(Logger::new(PathBuf::from("tmp/objeto"))?);
        let hash = HashObject {
            logger: logger.clone(),
            escribir: false,
            ubicacion_archivo: directorio.clone(),
        }
        .ejecutar()?;

        let nombre = obtener_nombre(&directorio)?;

        Ok(Blob {
            nombre,
            hash,
            ubicacion: directorio,
            logger,
        })
    }
}

impl Display for Blob {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string = format!("100644 {} {}\n", self.hash, self.ubicacion.display());
        write!(f, "{}", string)
    }
}

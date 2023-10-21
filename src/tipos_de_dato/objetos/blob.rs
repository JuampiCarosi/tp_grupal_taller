use crate::tipos_de_dato::comandos::cat_file::conseguir_tamanio;
use std::fmt::Display;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Blob {
    pub nombre: String,
    pub hash: String,
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
}

impl Display for Blob {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string = format!("100644 {} {}\n", self.hash, self.nombre);
        write!(f, "{}", string)
    }
}

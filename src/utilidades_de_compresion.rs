use std::io::{Read, Write};
use flate2::{read::ZlibDecoder, write::ZlibEncoder, Compression};
use crate::io;

pub fn descomprimir_objeto(hash: String) -> Result<String, String> {
    let ruta_objeto = format!(".gir/objects/{}/{}", &hash[..2], &hash[2..]);
    let contenido_leido = io::leer_bytes(&ruta_objeto)?;
    let mut descompresor = ZlibDecoder::new(contenido_leido.as_slice());
    let mut contenido_descomprimido = String::new();
    match descompresor.read_to_string(&mut contenido_descomprimido) {
        Ok(_) => {}
        Err(_) => {
            return Err(format!("No se pudo descomprimir el objeto {}", hash));
        }
    };
    Ok(contenido_descomprimido)
}

pub fn comprimir_contenido(contenido: String) -> Result<Vec<u8>, String> {
    let mut compresor = ZlibEncoder::new(Vec::new(), Compression::default());
    if compresor.write_all(contenido.as_bytes()).is_err() {
        return Err("No se pudo comprimir el contenido".to_string());
    };
    match compresor.finish() {
        Ok(contenido_comprimido) => Ok(contenido_comprimido),
        Err(_) => Err("No se pudo comprimir el contenido".to_string()),
    }
}

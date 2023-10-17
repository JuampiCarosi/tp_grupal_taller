use std::fs;

pub fn crear_directorio(directorio: &String) -> Result<(), String> {
    let dir = fs::metadata(directorio);
    if dir.is_ok() {
        return Ok(());
    }
    match fs::create_dir_all(directorio) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("Error al crear el directorio: {}", e)),
    }
}

pub fn leer_a_string(path: &String) -> Result<String, String> {
    match fs::read_to_string(path) {
        Ok(contenido) => Ok(contenido),
        Err(_) => {
            return Err("No se pudo leer el archivo".to_string());
        }
    }
}

pub fn escrbir_bytes(archivo: &String, contenido: Vec<u8>) -> Result<(), String> {
    let mut dir = archivo.split('/').collect::<Vec<&str>>();
    dir.pop();
    if !dir.is_empty() {
        crear_directorio(&dir.join("/"))?;
    }

    match fs::write(archivo, contenido) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("Error al escribir el archivo: {}", e)),
    }
}

pub fn leer_bytes(archivo: &String) -> Result<Vec<u8>, String> {
    match fs::read(archivo) {
        Ok(contenido) => Ok(contenido),
        Err(_) => {
            return Err("No se pudo leer el archivo".to_string());
        }
    }
}

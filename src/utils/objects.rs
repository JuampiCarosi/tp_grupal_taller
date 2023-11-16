use std::path::PathBuf;

use super::{io, path_buf};

///Devuelve todos los objetos dentro de objetcs (sus hash)
pub fn obtener_objetos() -> Result<Vec<String>, String> {
    let dir_abierto = io::leer_directorio(&"./.gir/objects/")?;

    let mut objetos: Vec<String> = Vec::new();

    for entrada in dir_abierto {
        match entrada {
            Ok(entrada) => {
                if io::es_dir(entrada.path())
                    && entrada.file_name().to_string_lossy() != "info"
                    && entrada.file_name().to_string_lossy() != "pack"
                {
                    //que onda este if?? JUANI
                    if !entrada.path().to_string_lossy().contains("log.txt") {
                        objetos.append(&mut obtener_objetos_con_nombre_carpeta(entrada.path())?);
                    }
                }
            }
            Err(error) => {
                return Err(format!("Error leyendo directorio: {}", error));
            }
        }
    }
    Ok(objetos)
}

///Obitiene todos los objetos asosiados a una carpeta dentro objetcs. Dado una carpeta, devuelve
/// todo los objtetos asosiados a este
///
/// ## Ejemplo
/// - recive: jk/
/// - devuleve: jksfsfsffafasfas...fdfdf, kjsfsfaftyhththht, jkiodf235453535355fs, ...
///
/// ## Error
/// -Si no existe dir
/// -Si no tiene conti8dio
pub fn obtener_objetos_con_nombre_carpeta(dir: PathBuf) -> Result<Vec<String>, String> {
    let directorio = io::leer_directorio(&dir)?;

    let mut objetos = Vec::new();
    let nombre_directorio = path_buf::obtener_nombre(&dir)?;

    for archivo in directorio {
        match archivo {
            Ok(archivo) => {
                objetos.push(
                    nombre_directorio.clone() + archivo.file_name().to_string_lossy().as_ref(),
                );
            }
            Err(error) => {
                return Err(format!("Error leyendo directorio: {}", error));
            }
        }
    }

    if objetos.is_empty() {
        return Err(format!(
            "Error el directorio {} no tiene cotenido",
            nombre_directorio
        ));
    }

    Ok(objetos)
}

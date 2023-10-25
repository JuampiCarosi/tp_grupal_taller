use std::{
    collections::HashSet,
    fs::{self, OpenOptions},
    io::{BufRead, Write},
    path::{Path, PathBuf},
    rc::Rc,
};

use crate::{
    tipos_de_dato::comandos::hash_object::HashObject, utilidades_path_buf::obtener_directorio_raiz, io,
};

use super::{logger::Logger, objeto::Objeto};

const PATH_INDEX:&str = "./.gir/index";

pub fn crear_index() {
    if Path::new(PATH_INDEX).exists() {
        return;
    }
    let _ = fs::File::create(PATH_INDEX);
}

//Devuelve true si el index esta vacio y false en caso contrario. 
//Si falla se presupone que es porque no existe y por lo tanto esta vacio 
pub fn esta_vacio_el_index() -> bool {
    let config_path = ".gir/CONFIG";
        let contenido = match io::leer_a_string(config_path) {
            Ok(contenido) => contenido,
            Err(_) => return true,
        };
        if contenido.is_empty() {
            return true;
        }
        false
}

pub fn leer_index() -> Result<Vec<Objeto>, String> {
    let file = match OpenOptions::new().read(true).open(PATH_INDEX) {
        Ok(file) => file,
        Err(_) => return Err("No se pudo abrir el archivo index".to_string()),
    };

    let mut objetos: Vec<Objeto> = Vec::new();

    for line in std::io::BufReader::new(file).lines() {
        if let Ok(line) = line.as_ref() {
            let objeto = Objeto::from_index(line.to_string())?;
            objetos.push(objeto);
        }
    }
    Ok(objetos)
}
pub fn generar_objetos_raiz(objetos: &Vec<Objeto>) -> Result<Vec<Objeto>, String> {
    let mut objetos_raiz: Vec<Objeto> = Vec::new();
    let mut directorios_raiz: HashSet<PathBuf> = HashSet::new();
    let mut directorios_a_tener_en_cuenta: Vec<PathBuf> = Vec::new();

    for objeto in objetos.iter() {
        match objeto {
            Objeto::Blob(blob) => {
                directorios_a_tener_en_cuenta.push(blob.ubicacion.clone());
                let padre = obtener_directorio_raiz(&blob.ubicacion)?;
                directorios_raiz.insert(PathBuf::from(padre));
            }
            Objeto::Tree(tree) => {
                if tree.es_vacio() {
                    continue;
                }
                directorios_a_tener_en_cuenta.extend(tree.obtener_paths_hijos());
                let padre = obtener_directorio_raiz(&tree.directorio)?;
                directorios_raiz.insert(PathBuf::from(padre));
            }
        }
    }

    for directorio in directorios_raiz {
        let objeto_conteniendo_al_blob =
            Objeto::from_directorio(directorio.clone(), Some(&directorios_a_tener_en_cuenta))?;

        objetos_raiz.push(objeto_conteniendo_al_blob);
    }

    objetos_raiz.sort_by_key(|x| match x {
        Objeto::Blob(blob) => blob.ubicacion.clone(),
        Objeto::Tree(tree) => PathBuf::from(&tree.directorio),
    });
    Ok(objetos_raiz)
}

pub fn escribir_index(logger: Rc<Logger>, objetos: &Vec<Objeto>) -> Result<(), String> {
    let mut file = match OpenOptions::new().write(true).open(PATH_INDEX) {
        Ok(file) => file,
        Err(_) => return Err("No se pudo escribir el archivo index".to_string()),
    };

    if objetos.is_empty() {
        OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(PATH_INDEX)
            .unwrap();
        return Ok(());
    }

    let mut buffer = String::new();

    for objeto in generar_objetos_raiz(objetos)? {
        let line = match objeto {
            Objeto::Blob(blob) => {
                HashObject {
                    logger: logger.clone(),
                    escribir: true,
                    ubicacion_archivo: blob.ubicacion.clone(),
                }
                .ejecutar()?;
                format!("{blob}")
            }
            Objeto::Tree(tree) => {
                tree.escribir_en_base()?;
                format!("{tree}")
            }
        };

        buffer.push_str(&line);
    }
    OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(PATH_INDEX)
        .unwrap();
    let _ = file.write_all(buffer.as_bytes());
    Ok(())
}

pub fn limpiar_archivo_index() -> Result<(), String> {
    let _ = match OpenOptions::new()
        .write(true)
        .truncate(true)
        .open("./.gir/index") {
            Ok(archivo) => archivo,
            Err(_) => return Err("No se pudo abrir el archivo index".to_string()),
        };
    Ok(())    
}

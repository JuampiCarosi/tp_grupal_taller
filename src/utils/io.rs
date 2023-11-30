use crate::err_comunicacion::ErrorDeComunicacion;
use std::fmt::Debug;
use std::fs::{self, File, ReadDir};
use std::io::{self, BufRead};
use std::path::{Path, PathBuf};
use std::{env, str};

use super::path_buf;

//la idea es dejar de usar esta funcion, ->ya hay una mejor en objects
pub fn obtener_objetos_del_directorio(dir: &str) -> Result<Vec<String>, String> {
    let dir_abierto = leer_directorio(&dir)?;

    let mut objetos: Vec<String> = Vec::new();

    for entrada in dir_abierto {
        match entrada {
            Ok(entrada) => {
                if es_dir(entrada.path())
                    && entrada.file_name().to_string_lossy() != "info"
                    && entrada.file_name().to_string_lossy() != "pack"
                {
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

// dado un directorio devuelve el nombre del archivo contenido (solo caso de objectos de git)
pub fn obtener_objetos(dir: PathBuf) -> Result<String, ErrorDeComunicacion> {
    let mut directorio = fs::read_dir(dir.clone())?;
    if let Some(archivo) = directorio.next() {
        match archivo {
            Ok(archivo) => {
                return Ok(archivo.file_name().to_string_lossy().to_string());
            }
            Err(error) => {
                eprintln!("Error leyendo directorio: {}", error);
            }
        }
    }
    Err(ErrorDeComunicacion::IoError(io::Error::new(
        io::ErrorKind::NotFound,
        "Hubo un error al obtener el objeto",
    )))
}

///Obitiene todos los objetos asosiados a una carpeta dentro dir. Dado una carpeta, devuelve
/// todo los objtetos asosiados a este
///
/// ## Ejemplo
/// - recive: jk/
/// - devuleve: jksfsfsffafasfas...fdfdf, kjsfsfaftyhththht, jkiodf235453535355fs, ...
///
/// ## Error
/// -Si no existe dir
/// -Si no tiene contendio
pub fn obtener_objetos_con_nombre_carpeta(dir: PathBuf) -> Result<Vec<String>, String> {
    let directorio = leer_directorio(&dir)?;

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

pub fn obtener_refs_con_largo_hex(
    refs: &mut Vec<String>,
    refs_path: PathBuf,
    dir: &str,
) -> Result<(), ErrorDeComunicacion> {
    // let mut refs: Vec<String> = Vec::new();
    if !refs_path.exists() {
        return Ok(());
    }
    // if !refs_path.exists() {
    //     io::Error::new(io::ErrorKind::NotFound, "No existe el repositorio");
    // }
    let head_dir = fs::read_dir(&refs_path)?;
    for archivo in head_dir {
        match archivo {
            Ok(archivo) => {
                let mut path = archivo.path();
                let referencia =
                    obtener_linea_con_largo_hex(obtener_referencia(&mut path, dir)?.as_str());
                refs.push(referencia);
                // println!("Obtengo la ref: {}", referencia);
            }
            Err(error) => {
                eprintln!("Error leyendo directorio: {}", error);
            }
        }
    }
    Ok(())
}

pub fn obtener_refs(refs_path: PathBuf, dir: &str) -> Result<Vec<String>, ErrorDeComunicacion> {
    let mut refs: Vec<String> = Vec::new();
    if !refs_path.exists() {
        return Ok(refs);
        // io::Error::new(io::ErrorKind::NotFound, "No existe el repositorio");
    }

    if refs_path.ends_with("HEAD") {
        refs.push(obtener_ref_head(refs_path.to_path_buf())?);
    } else {
        let head_dir = fs::read_dir(&refs_path)?;
        for archivo in head_dir {
            match archivo {
                Ok(archivo) => {
                    let mut path = archivo.path();
                    // let mut path = archivo.path().to_string_lossy().split("./.gir/").into_iter().next().unwrap().to_string();
                    refs.push(obtener_referencia(&mut path, dir)?);
                }
                Err(error) => {
                    eprintln!("Error leyendo directorio: {}", error);
                }
            }
        }
    }
    Ok(refs)
}

pub fn calcular_largo_hex(line: &str) -> String {
    let largo = line.len() + 4; // el + 4 es por los 4 bytes que indican el largo
    let largo_hex = format!("{:x}", largo);
    format!("{:0>4}", largo_hex)
}

pub fn obtener_linea_con_largo_hex(line: &str) -> String {
    let largo_hex = calcular_largo_hex(line);
    format!("{}{}", largo_hex, line)
}

fn leer_archivo(path: &mut Path) -> Result<String, ErrorDeComunicacion> {
    let archivo = fs::File::open(path)?;
    let mut contenido = String::new();
    std::io::BufReader::new(archivo).read_line(&mut contenido)?;
    Ok(contenido.trim().to_string())
}
//Devuelve true si la ubicacion esta vacia y false en caso contrario.
//Si falla se presupone que es porque no existe y por lo tanto esta vacio
pub fn esta_vacio(ubicacion: &str) -> bool {
    match fs::metadata(ubicacion) {
        Ok(metadata) => metadata.len() == 0,
        Err(_) => false,
    }
}

fn obtener_referencia(path: &mut Path, prefijo: &str) -> Result<String, ErrorDeComunicacion> {
    let contenido = leer_archivo(path)?;
    // esto esta hardcodeado, hay que cambiar la forma de sacarle el prefijo
    let directorio_sin_prefijo = path.strip_prefix(prefijo).unwrap().to_path_buf();
    let referencia = format!(
        "{} {}",
        contenido.trim(),
        directorio_sin_prefijo.to_str().ok_or(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "No existe HEAD"
        ))?
    );
    Ok(referencia)
}

pub fn obtener_ref_head(path: PathBuf) -> Result<String, ErrorDeComunicacion> {
    if !path.exists() {
        return Err(ErrorDeComunicacion::IoError(io::Error::new(
            io::ErrorKind::NotFound,
            "No existe HEAD",
        )));
    }
    let contenido = leer_archivo(&mut path.clone())?;
    let head_ref = contenido.split_whitespace().collect::<Vec<&str>>()[1];
    if let Some(ruta) = path.clone().parent() {
        let cont = leer_archivo(&mut ruta.join(head_ref))? + " HEAD";
        Ok(obtener_linea_con_largo_hex(&cont))
    } else {
        Err(ErrorDeComunicacion::IoError(io::Error::new(
            io::ErrorKind::NotFound,
            "No existe HEAD",
        )))
    }
}

///Lee un directorio. Devuelve su iterador. Falla si no existe o si no es un directoro
pub fn leer_directorio<P: AsRef<Path> + Clone + Debug>(directorio: &P) -> Result<ReadDir, String> {
    let metadada_dir =
        fs::metadata(directorio).map_err(|_| format!("Error no existe el dir {:?}", directorio))?;

    if !metadada_dir.is_dir() {
        return Err(format!("Error {:?} no es un dir", directorio));
    }

    fs::read_dir(directorio).map_err(|e| format!("Error al leer {:?}: {}", directorio, e))
}

///Devuelve True si el directororio es un directorio o false en caso contrario o si no existe
pub fn es_dir<P: AsRef<Path> + Clone + Debug>(entrada: P) -> bool {
    match fs::metadata(entrada) {
        Ok(metadata_contenido) => metadata_contenido.is_dir(),
        Err(_) => false,
    }
}

///Crea un
pub fn crear_directorio<P: AsRef<Path> + Clone>(directorio: P) -> Result<(), String> {
    let dir = fs::metadata(directorio.clone());
    if dir.is_ok() {
        return Ok(());
    }
    match fs::create_dir_all(directorio) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("Error al crear el directorio: {}", e)),
    }
}
///Similar a `crear_directorio` pero puede fallar si la carpeta ya existe
pub fn crear_carpeta<P: AsRef<Path> + Clone>(carpeta: P) -> Result<(), String> {
    match fs::create_dir_all(carpeta) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("Error al crear la carpeta: {}", e)),
    }
}

pub fn cambiar_directorio<P: AsRef<Path> + Clone + Debug>(directorio: P) -> Result<(), String> {
    env::set_current_dir(&directorio)
        .map_err(|err| format!("Fallo al cambiar de directorio {:?}:{}", directorio, err))
}
pub fn crear_archivo<P: AsRef<Path> + Clone>(dir_directorio: P) -> Result<(), String> {
    si_no_existe_directorio_de_archivo_crearlo(&dir_directorio)?;
    if !dir_directorio.as_ref().exists() {
        File::create(dir_directorio.clone()).map_err(|err| format!("{}", err))?;
    }

    Ok(())
}

pub fn leer_a_string<P>(path: P) -> Result<String, String>
where
    P: AsRef<Path>,
{
    match fs::read_to_string(&path) {
        Ok(contenido) => Ok(contenido),
        Err(_) => Err(format!(
            "No se pudo leer el archivo {}",
            path.as_ref().display()
        )),
    }
}

pub fn escribir_bytes<P, C>(dir_archivo: P, contenido: C) -> Result<(), String>
where
    P: AsRef<Path>,
    C: AsRef<[u8]>,
{
    si_no_existe_directorio_de_archivo_crearlo(&dir_archivo)?;

    match fs::write(dir_archivo, contenido) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("Error al escribir el archivo: {}", e)),
    }
}

pub fn leer_bytes<P>(archivo: P) -> Result<Vec<u8>, String>
where
    P: AsRef<Path>,
{
    match fs::read(&archivo) {
        Ok(contenido) => Ok(contenido),
        Err(_) => Err(format!(
            "No se pudo leer el archivo leyendo bytes {}",
            archivo.as_ref().display()
        )),
    }
}

pub fn si_no_existe_directorio_de_archivo_crearlo<P>(dir_archivo: &P) -> Result<(), String>
where
    P: AsRef<Path>,
{
    let dir = dir_archivo.as_ref().parent();
    if let Some(parent_dir) = dir {
        let parent_str = parent_dir
            .to_str()
            .ok_or_else(|| String::from("Error al convertir el directorio a cadena"))?;

        crear_directorio(parent_str.to_owned() + "/")?;
    };
    Ok(())
}

pub fn rm_directorio<P>(directorio: P) -> Result<(), String>
where
    P: AsRef<Path>,
{
    let metadata = fs::metadata(&directorio).map_err(|_| {
        format!(
            "No se pudo borrar el directorio {}",
            directorio.as_ref().display()
        )
    })?;

    if metadata.is_file() {
        return match fs::remove_file(&directorio) {
            Ok(_) => Ok(()),
            Err(_) => Err(format!(
                "No se pudo borrar el directorio {}",
                directorio.as_ref().display()
            )),
        };
    }

    if metadata.is_dir() {
        return match fs::remove_dir_all(&directorio) {
            Ok(_) => Ok(()),
            Err(_) => Err(format!(
                "No se pudo borrar el directorio {}",
                directorio.as_ref().display()
            )),
        };
    }
    Err(format!(
        "No se pudo borrar el directorio {}",
        directorio.as_ref().display()
    ))
}

// dado un vector con nombres de archivos de vuelve aquellos que no estan en el directorio

// HACER MAS EFICIENTE *Hay iteraciones de mas que se pueden evitar unificando las funciones*
pub fn obtener_archivos_faltantes(nombres_archivos: Vec<String>, dir: String) -> Vec<String> {
    // DESHARDCODEAR EL NOMBRE DEL DIRECTORIO (.gir)
    let objetcts_contained = obtener_objetos_del_directorio(&(dir + "objects/")).unwrap();
    // println!("objetcts_contained: {:?}", objetcts_contained);
    // println!("Nombres: {:?}", nombres_archivos);
    let mut archivos_faltantes: Vec<String> = Vec::new();
    for nombre in &objetcts_contained {
        if nombres_archivos.contains(nombre) {
        } else {
            archivos_faltantes.push(nombre.clone());
        }
    }
    archivos_faltantes
}
// aca depende de si esta multi_ack y esas cosas, esta es para cuando no hay multi_ack ni multi_ack_mode
pub fn obtener_ack(nombres_archivos: Vec<String>, dir: &str) -> Vec<String> {
    let mut ack = Vec::new();
    for nombre in nombres_archivos {
        let dir_archivo = format!("{}{}/{}", dir.clone(), &nombre[..2], &nombre[2..]);
        if PathBuf::from(dir_archivo.clone()).exists() {
            ack.push(obtener_linea_con_largo_hex(
                ("ACK".to_string() + &nombre + "\n").as_str(),
            ));
            break;
        }
    }
    if ack.is_empty() {
        ack.push(obtener_linea_con_largo_hex("NAK\n"));
    }
    ack
}

// las referencias vienen en formato "hash referencia"
pub fn escribir_referencia(referencia: &str, dir: PathBuf) {
    let referencia_y_contenido = referencia.split_whitespace().collect::<Vec<&str>>();
    if !&referencia_y_contenido[1].contains("HEAD") {
        let dir = dir.join(referencia_y_contenido[1]);
        escribir_bytes(dir, referencia_y_contenido[0]).unwrap();
    }
}

pub fn obtener_diferencias_remote(referencias: Vec<String>, dir: String) -> Vec<String> {
    let mut diferencias: Vec<String> = Vec::new();
    // si no existe devuelvo todas las refs
    if !PathBuf::from(dir.clone() + "refs/remotes/origin/").exists() {
        return referencias;
    }
    for referencia in referencias {
        let referencia_y_contenido = referencia.split_whitespace().collect::<Vec<&str>>();
        let referencia_remote = "refs/remotes/origin/".to_string()
            + referencia_y_contenido[1].split('/').last().unwrap();
        let referencia_local =
            leer_a_string(Path::new(&(dir.clone() + &referencia_remote))).unwrap();
        if referencia_local != referencia_y_contenido[0] {
            diferencias.push(referencia_y_contenido[0].to_string());
        }
    }
    diferencias
}

mod tests {

    use super::*;

    #[test]
    fn test_escribir_archivo_pisa_contenido() {
        let dir = PathBuf::from("tmp/test_escribir_archivo_pisa_contenido.txt");
        escribir_bytes(&dir, "contenido 1").unwrap();
        escribir_bytes(&dir, "contenido 2").unwrap();
        assert_eq!(leer_a_string(&dir).unwrap(), "contenido 2");
        rm_directorio(dir).unwrap();
    }
}

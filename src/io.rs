use std::io::{self, BufRead};
use std::path::{PathBuf, Path};
use std::fs::{self, File};
use std::str;
use crate::err_comunicacion::ErrorDeComunicacion;

pub fn leer_archivos_directorio(direccion: &mut Path) -> Result<Vec<String>, ErrorDeComunicacion>{
    let mut contenidos: Vec<String> = Vec::new();
    let head_dir = fs::read_dir(&direccion)?;
    for archivo in head_dir {
        match archivo {
            Ok(archivo) => {
                let path = archivo.path();
                contenidos.push(obtener_referencia(&mut path.clone())?);
            }
            Err(error) => {
                eprintln!("Error leyendo directorio: {}", error);
            }
        }
    }
    Ok(contenidos)
}

pub fn obtener_objetos_del_directorio(dir: String) -> Result<Vec<String>, ErrorDeComunicacion>{
    let path = PathBuf::from(dir);
    println!("path: {:?}", path);
    let mut objetos: Vec<String> = Vec::new();
    
    let dir_abierto = fs::read_dir(path.clone())?;
    
    for archivo in  dir_abierto {
        match archivo {
            Ok(archivo) => {
                if archivo.file_type().unwrap().is_dir() && archivo.file_name().into_string().unwrap() != "info" && archivo.file_name().into_string().unwrap() != "pack"{
                    let path = archivo.path();
                    let nombre_carpeta = archivo.file_name().into_string().unwrap();
                    objetos.push(format!("{}{}", nombre_carpeta, obtener_objeto(path.clone())?));
                }
            }
            Err(error) => {
                eprintln!("Error leyendo directorio: {}", error);
            }
        }
    }
    Ok(objetos)
}

// dado un directorio devuelve el nombre del archivo contenido (solo caso de objectos de git)
pub fn obtener_objeto(dir: PathBuf) -> Result<String, ErrorDeComunicacion>{
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
    println!("no hay archivos en el directorio: {:?}", dir);
    Err(ErrorDeComunicacion::IoError(io::Error::new(io::ErrorKind::NotFound, "Hubo un error al obtener el objeto")))
}


pub fn obtener_refs(path: &mut Path) -> Result<Vec<String>, ErrorDeComunicacion> {
    let mut refs: Vec<String> = Vec::new();
    if !path.exists() {
        io::Error::new(io::ErrorKind::NotFound, "No existe el repositorio");
    }
    if path.ends_with("HEAD") {
        refs.push(obtener_ref_head(path.to_path_buf())?);
    }
    else {
        let head_dir = fs::read_dir(&path)?;
        for archivo in head_dir {
            match archivo {
                Ok(archivo) => {
                    let path = archivo.path();
                    refs.push(obtener_referencia(&mut path.clone())?);
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
    let archivo = fs::File::open(path.to_path_buf())?;
    let mut contenido = String::new();            
    std::io::BufReader::new(archivo).read_line(&mut contenido)?;
    Ok(contenido.trim().to_string())
}

fn obtener_referencia(path: &mut Path) -> Result<String, ErrorDeComunicacion> {
    let contenido = leer_archivo(path)?;            
    let referencia = format!("{} {}", contenido.trim(), path.to_str().ok_or(std::io::Error::new(std::io::ErrorKind::NotFound, "No existe HEAD"))?);
    Ok(obtener_linea_con_largo_hex(&referencia))
}

fn obtener_ref_head(path: PathBuf) -> Result<String, ErrorDeComunicacion>{
    if !path.exists() {
        return Err(ErrorDeComunicacion::IoError(io::Error::new(io::ErrorKind::NotFound, "No existe HEAD")));
    }
    let contenido = leer_archivo(&mut path.clone())?;
    let head_ref: Vec<&str> = contenido.split_whitespace().collect();
    if let Some(ruta) = path.clone().parent(){
        let cont = leer_archivo(&mut ruta.join(head_ref[1]))? + " HEAD";
        Ok(obtener_linea_con_largo_hex(&cont))
    } else {
        Err(ErrorDeComunicacion::IoError(io::Error::new(io::ErrorKind::NotFound, "No existe HEAD")))
    }
}


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
fn si_no_existe_directorio_de_archivo_crearlo<P>(dir_archivo: &P) -> Result<(), String>
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

use std::io::{self, Write, Read, BufRead};
use std::path::PathBuf;
use std::fs;
use std::str;
use crate::err_comunicacion::ErrorDeComunicacion;


pub fn leer_archivos_directorio(direccion: &mut PathBuf) -> Result<Vec<String>, ErrorDeComunicacion>{
    let mut contenidos: Vec<String> = Vec::new();
    let head_dir = fs::read_dir(direccion.clone())?;
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


pub fn obtener_refs(path: &mut PathBuf) -> Result<Vec<String>, ErrorDeComunicacion> {
    let mut refs: Vec<String> = Vec::new();
    if !path.exists() {
        ErrorDeComunicacion::IoError(io::Error::new(io::ErrorKind::NotFound, "No existe el repositorio"));
    }
    if path.ends_with("HEAD") {
        refs.push(obtener_ref_head(path.clone())?);
    }
    else {
        let head_dir = fs::read_dir(path.clone())?;
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

fn leer_archivo(path: &mut PathBuf) -> Result<String, ErrorDeComunicacion> {
    let archivo = fs::File::open(path.clone())?;
    let mut contenido = String::new();            
    std::io::BufReader::new(archivo).read_line(&mut contenido)?;
    Ok(contenido.trim().to_string())
}

fn obtener_referencia(path: &mut PathBuf) -> Result<String, ErrorDeComunicacion> {
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
        return Ok(obtener_linea_con_largo_hex(&cont));
    } else {
        return Err(ErrorDeComunicacion::IoError(io::Error::new(io::ErrorKind::NotFound, "No existe HEAD")));
    }
}


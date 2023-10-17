use std::io::{self, Write, Read, BufRead};
use std::path::PathBuf;
use std::fs;
use std::str;
use crate::err_comunicacion::ErrorDeComunicacion;


pub fn get_refs(path: &mut PathBuf) -> Result<Vec<String>, ErrorDeComunicacion> {
    let mut refs: Vec<String> = Vec::new();
    if !path.exists() {
        ErrorDeComunicacion::IoError(io::Error::new(io::ErrorKind::NotFound, "No existe el repositorio"));
    }
    if path.ends_with("HEAD") {
        refs.push(get_head_ref(path.clone())?);
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

fn leer_archivo(path: &mut PathBuf) -> Result<String, ErrorDeComunicacion> {
    let archivo = fs::File::open(path.clone())?;
    let mut contenido = String::new();            
    std::io::BufReader::new(archivo).read_line(&mut contenido)?;
    Ok(contenido)
}

fn obtener_referencia(path: &mut PathBuf) -> Result<String, ErrorDeComunicacion> {
    let contenido = leer_archivo(path)?;            
    let referencia = format!("{} {}", contenido.trim(), path.to_str().ok_or(std::io::Error::new(std::io::ErrorKind::NotFound, "No existe HEAD"))?);
    Ok(format!("{}{}", calcular_largo_hex(&referencia).as_str(), referencia))
}

fn get_head_ref(path: PathBuf) -> Result<String, ErrorDeComunicacion>{
    if !path.exists() {
        return Err(ErrorDeComunicacion::IoError(io::Error::new(io::ErrorKind::NotFound, "No existe HEAD")));
    }
    let archivo = fs::File::open(path.clone())?;
    let mut contenido = String::new();            
    std::io::BufReader::new(archivo).read_line(&mut contenido)?;
    let head_ref: Vec<&str> = contenido.split_whitespace().collect();
    if let Some(ruta) = path.clone().parent(){
        return Ok(obtener_referencia(&mut ruta.join(head_ref[1]))?);
    } else {
        return Err(ErrorDeComunicacion::IoError(io::Error::new(io::ErrorKind::NotFound, "No existe HEAD")));
    }
}


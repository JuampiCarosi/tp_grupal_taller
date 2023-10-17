use std::net::{TcpListener, TcpStream};
use std::io::{self, Write, Read, BufRead};
use std::env;
use std::path::PathBuf;
use std::str;
use crate::err_comunicacion::ErrorDeComunicacion;
use crate::io as git_io;

pub struct Comunicacion { 
    listener: TcpListener,
    stream: TcpStream,
    dir: String,  
}

impl Comunicacion { 

    pub fn new(listener: TcpListener, dir: String) -> Comunicacion {
        let (stream, _) = listener.accept().unwrap();
        Comunicacion { listener, stream, dir }
    }

    

    pub fn procesar_datos(&mut self) -> Result<(), ErrorDeComunicacion> {
        // lee primera parte, 4 bytes en hexadecimal indican el largo del stream
        let mut length_bytes = [0; 4];
        self.stream.read_exact(&mut length_bytes)?;
        // largo de bytes a str
        let length_str = str::from_utf8(&length_bytes)?;
        // transforma str a u32
        let length = u32::from_str_radix(length_str, 16).unwrap();
        println!("length: {:?}", length);

        // lee el resto del stream
        let mut data = vec![0; (length - 4) as usize];
        self.stream.read_exact(&mut data)?;
        let line = str::from_utf8(&data)?;
        println!("Received: {:?}", line);
        // println!("length: {:?}", Self::calcular_largo_hex(&line));
        self.parse_line(&line);
    //     // stream.write(line.as_bytes())?;
        Ok(())
    }
    // pub fn calcular_largo_hex(line: &str) -> String {
    //     let largo = line.len() + 4; // el + 4 es por los 4 bytes que indican el largo
    //     let largo_hex = format!("{:x}", largo);
    //     format!("{:0>4}", largo_hex)
    // }

    fn parse_line(&mut self, line: &str) -> Result<(), ErrorDeComunicacion>{
        let req: Vec<&str> = line.split_whitespace().collect();
        // veo si es un comando git
        match req[0] {
            "git-upload-pack" => {
                println!("git-upload-pack");
                let args: Vec<_> = req[1].split('\0').collect();    
                let path = PathBuf::from(self.dir.clone() + args[0]);
                let mut refs: Vec<String> = Vec::new();
                refs.append(&mut git_io::obtener_refs(&mut path.clone().join("HEAD"))?);
                refs.append(&mut git_io::obtener_refs(&mut path.join("refs/heads/"))?);
                refs.append(&mut git_io::obtener_refs(&mut path.join("refs/tags/"))?);
                println!("refs: {:?}", refs);
                Ok(())
            },
            _ =>    {
                println!("No se reconoce el comando");
                Ok(())
            },
        }
    }
    
    // fn get_refs(&mut self, path: &mut PathBuf) -> Result<(), ErrorDeComunicacion> {
    //     if path.ends_with("HEAD") {
    //         let _ = self.get_head_ref(path.clone())?;
    //     }
    //     else {
    //         let head_dir = fs::read_dir(path.clone())?;
    //         for archivo in head_dir {
    //             match archivo {
    //                 Ok(archivo) => {
    //                     let path = archivo.path();
    //                     self.obtener_referencia(&mut path.clone())?;
    //                 }
    //                 Err(error) => {
    //                     eprintln!("Error leyendo directorio: {}", error);
    //                 }
    //             }
    //         }
    //     }
    //     Ok(())
    // }

    // fn obtener_referencia(&mut self, path: &mut PathBuf) -> Result<(), ErrorDeComunicacion> {
    //     let archivo = fs::File::open(path.clone())?;
    //     let mut contenido = String::new();            
    //     std::io::BufReader::new(archivo).read_line(&mut contenido)?;
    //     let referencia = format!("{} {}", contenido.trim(), path.to_str().ok_or(std::io::Error::new(std::io::ErrorKind::NotFound, "No existe HEAD"))?);
    //     println!("{}", format!("{}{}", Self::calcular_largo_hex(&referencia).as_str(), referencia));
    //     Ok(())
    // }

    // fn get_head_ref(&mut self, path: PathBuf) -> Result<(), ErrorDeComunicacion>{
    //     if !path.exists() {
    //         return Err(ErrorDeComunicacion::IoError(io::Error::new(io::ErrorKind::NotFound, "No existe HEAD")));
    //     }
    //     let archivo = fs::File::open(path)?;
    //     let mut contenido = String::new();            
    //     std::io::BufReader::new(archivo).read_line(&mut contenido)?;
    //     let head_ref: Vec<&str> = contenido.split_whitespace().collect();
    //     self.obtener_referencia(&mut PathBuf::from(self.dir.clone() + "/" + head_ref[1]))?;
    //     Ok(())
    // }
}
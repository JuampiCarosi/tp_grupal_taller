use crate::{io, tipos_de_dato::logger::Logger};
use flate2::{self, write::ZlibEncoder, Compression};
use sha1::{Digest, Sha1};
use std::{io::Write, rc::Rc};

pub struct HashObject {
    logger: Rc<Logger>,
    escribir: bool,
    nombre_archivo: String,
}

impl HashObject {
    fn obtener_nombre_archivo(args: &mut Vec<String>) -> Result<String, String> {
        match args.pop() {
            Some(nombre_archivo) => Ok(nombre_archivo),
            None => Err("No se especifico un archivo".to_string()),
        }

        //args.pop().ok_or_else(|| "No se especifico un archivo".to_string())
    }

    pub fn from(args: &mut Vec<String>, logger: Rc<Logger>) -> Result<HashObject, String> {
        let mut escribir = false;
        let nombre_archivo = Self::obtener_nombre_archivo(args)?;

        let iterador = args.iter();
        for arg in iterador {
            match arg.as_str() {
                "-w" => {
                    escribir = true;
                }
                _ => {
                    return Err(format!(
                        "Opcion desconocida {}\n gir hash-object [-w] <file>",
                        arg
                    ));
                }
            }
        }
        Ok(HashObject {
            logger,
            nombre_archivo,
            escribir,
        })
    }

    // fn hashear_objeto(&self) -> Result<(String, String), String> {
    //     let contenido = self.construir_contenido()?;
    //     let hash = self.hashear_contenido_objeto(&contenido);
    //     Ok((hash, contenido))
    // }

    fn construir_contenido(&self) -> Result<String, String> {
        let contenido = io::leer_a_string(&self.nombre_archivo.clone())?;
        let header = format!("blob {}\0", contenido.len());
        let contenido_total = header + &contenido;
        Ok(contenido_total)
    }

    fn hashear_contenido_objeto(&self, contenido: &str) -> String {
        let mut hasher = Sha1::new();
        hasher.update(contenido);
        let hash = hasher.finalize();
        format!("{:x}", hash)
    }

    fn comprimir_contenido(&self, contenido: String) -> Result<Vec<u8>, String> {
        let mut compresor = ZlibEncoder::new(Vec::new(), Compression::default());
        if compresor.write_all(contenido.as_bytes()).is_err() {
            return Err("No se pudo comprimir el contenido".to_string());
        };
        match compresor.finish() {
            Ok(contenido_comprimido) => Ok(contenido_comprimido),
            Err(_) => Err("No se pudo comprimir el contenido".to_string()),
        }
    }

    pub fn ejecutar(&self) -> Result<String, String> {
        let contenido = self.construir_contenido()?;
        let hash = self.hashear_contenido_objeto(&contenido);
        
        println!("{}", hash);
        if self.escribir {
            let ruta = format!(".gir/objects/{}/{}", &hash[..2], &hash[2..]);
            io::escrbir_bytes(&ruta, self.comprimir_contenido(contenido)?)?;
        }
        let mensaje = format!("Objeto gir hasheado en {}", self.nombre_archivo);
        self.logger.log(mensaje);
        Ok(hash)
    }
}

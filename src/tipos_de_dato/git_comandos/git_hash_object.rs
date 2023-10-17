use std::{fs, path::Path, rc::Rc};
use crate::flate2;
use crate::tipos_de_dato::logger::Logger;

pub struct GitHashObject {
    logger: Rc<Logger>,
    tipo_objeto: String,
    escribir: bool,
    ruta: String,
}

impl GitHashObject {
    pub fn new(args: Vec<String>, logger: Rc<Logger>) -> Result<GitHashObject, String> {
        let mut tipo_objeto = String::from("blob");
        let mut prefijo = String::from("./");
        let nombre_archivo = args.pop().unwrap();
        let mut escribir = false;
        for arg in args {
            match arg {
                "-t" => {
                    tipo_objeto = args.next();
                }
                "-w" => {
                    escribir = true;
                }
                "--path" => {
                    prefix = args.next();
                }
                _ => {
                    return Err(format!("Opcion desconocida {}, gir hash-object [-t <type>] [-w] [--path=<file>] <file>", arg));
                }
            }
        }
        let ruta = path::new(&prefijo).join(&nombre_archivo);
        Ok(GitHashObject{ logger, tipo_objeto, escribir, ruta })
    }

    pub fn hash_object(&self) -> String {
        let mensaje = format!("Objeto gir hasheado en {}", self.ruta);
        let contenido = fs::read_to_string(self.ruta).unwrap();
        let hash = self.hash_object_content(&contenido);
        self.logger.log(mensaje);
        hash
    }

    pub fn hash_object_content(&self, contenido: &str) -> String {
        let header = format!("{} #\{{}\}\0", self.tipo_objeto, contenido.len());
        let contenido_total = header + contenido;
        let hash = sha1::Sha1::from(contenido_total).hexdigest();
        hash
    }

    pub fn ejecutar(&self) {
        let hash = self.hash_object();
        if self.escribir {
            let ruta = path::new(".gir/objects").join(&hash[..2]).join(&hash[2..]);
            fs::create_dir_all(ruta.parent().unwrap()).unwrap();
            let contenido = fs::read_to_string(self.ruta).unwrap();
            fs::write(ruta, flate2::compress(&contenido_total)).unwrap();
        }
    }
}



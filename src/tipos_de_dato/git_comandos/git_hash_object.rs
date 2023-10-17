use crate::tipos_de_dato::logger::Logger;
use flate2::{self, write::ZlibEncoder, Compression};
use sha1::{Digest, Sha1};
use std::{fs, io::Write, path, rc::Rc};

pub struct GitHashObject {
    logger: Rc<Logger>,
    tipo_objeto: String,
    escribir: bool,
    ruta: String,
}

impl GitHashObject {
    pub fn from(mut args: Vec<String>, logger: Rc<Logger>) -> Result<GitHashObject, String> {
        let mut tipo_objeto = String::from("blob");
        let mut prefijo = String::from("./");
        let nombre_archivo = args.pop().unwrap();
        let mut escribir = false;
        let mut iterador = args.iter();
        while let Some(arg) = iterador.next() {
            match arg.as_str() {
                "-t" => {
                    tipo_objeto = iterador.next().unwrap().to_string();
                }
                "-w" => {
                    escribir = true;
                }
                "--path" => {
                    prefijo = iterador.next().unwrap().to_string();
                }
                _ => {
                    return Err(format!("Opcion desconocida {}, gir hash-object [-t <type>] [-w] [--path=<file>] <file>", arg));
                }
            }
        }
        let ruta = format!("{}/{}", prefijo, nombre_archivo);
        Ok(GitHashObject {
            logger,
            tipo_objeto,
            escribir,
            ruta,
        })
    }

    pub fn hashear_objeto(&self) -> String {
        let mensaje = format!("Objeto gir hasheado en {}", self.ruta);
        let contenido = fs::read_to_string(self.ruta.clone()).unwrap();
        let hash = self.hashear_contenido_objeto(&contenido);
        self.logger.log(mensaje);
        hash
    }

    pub fn hashear_contenido_objeto(&self, contenido: &str) -> String {
        let header = format!("{} #`{{`{}`}}`\0", self.tipo_objeto, contenido.len());
        let contenido_total = header + contenido;
        let mut hasher = Sha1::new();
        hasher.update(contenido_total);
        let hash = hasher.finalize();
        format!("{:x}", hash)
    }

    pub fn ejecutar(&self) -> Result<(), String> {
        let hash = self.hashear_objeto();
        println!("{}", hash);
        if self.escribir {
            let ruta = path::Path::new(".gir/objects")
                .join(&hash[..2])
                .join(&hash[2..]);
            fs::create_dir_all(ruta.parent().unwrap()).unwrap();
            let contenido = fs::read_to_string(self.ruta.clone()).unwrap();
            let mut compresor = ZlibEncoder::new(Vec::new(), Compression::default());
            compresor.write_all(contenido.as_bytes()).unwrap();
            let bytes_comprimidos = compresor.finish().unwrap();
            fs::write(ruta, bytes_comprimidos).unwrap();
        }
        Ok(())
    }
}

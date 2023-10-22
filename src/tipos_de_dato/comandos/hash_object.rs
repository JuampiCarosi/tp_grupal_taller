use crate::{io, tipos_de_dato::logger::Logger};
use flate2::{self, write::ZlibEncoder, Compression};
use sha1::{Digest, Sha1};
use std::{fs, io::Write, path, rc::Rc};

pub struct HashObject {
    logger: Rc<Logger>,
    tipo_objeto: String,
    escribir: bool,
    ruta: String,
}

impl HashObject {
    fn obtener_nombre_archivo(args: &mut Vec<String>) -> Result<String, String> {
        match args.pop() {
            Some(nombre_archivo) => {
                if !path::Path::new(&nombre_archivo).exists() {
                    return Err(format!("No existe el archivo {}", nombre_archivo));
                }
                if !path::Path::new(&nombre_archivo).is_file() {
                    return Err(format!("{} no es un archivo", nombre_archivo));
                }
                Ok(nombre_archivo)
            }
            None => {
                return Err(format!("No se especifico un archivo"));
            }
        }
    }

    fn escribir_tipo<'a>(
        siguiente: Option<&'a String>,
        tipo: &'a mut String,
    ) -> Result<(), String> {
        let tipo_a_escribir = match siguiente {
            Some(tipo) => tipo,
            None => {
                return Err(format!("Se esperaba un tipo de objeto luego de -t"));
            }
        };

        match tipo_a_escribir.as_str() {
            "blob" | "commit" | "tree" | "tag" => *tipo = tipo_a_escribir.to_owned(),
            _ => return Err(format!("Tipo de objeto invalido {}", tipo)),
        };

        Ok(())
    }

    fn escribir_path<'a>(
        siguiente: Option<&'a String>,
        path: &'a mut String,
    ) -> Result<(), String> {
        let path_a_escribir = match siguiente {
            Some(path) => path,
            None => {
                return Err(format!("Se esperaba un path luego de --path"));
            }
        };

        *path = path_a_escribir.to_owned();

        Ok(())
    }

    pub fn from(args: &mut Vec<String>, logger: Rc<Logger>) -> Result<HashObject, String> {
        let mut tipo_objeto = String::from("blob");
        let mut prefijo = String::from("./");
        let mut escribir = false;
        let nombre_archivo = Self::obtener_nombre_archivo(args)?;

        let mut iterador = args.iter();
        while let Some(arg) = iterador.next() {
            match arg.as_str() {
                "-t" => Self::escribir_tipo(iterador.next(), &mut tipo_objeto)?,
                "-w" => {
                    escribir = true;
                }
                "--path" => Self::escribir_path(iterador.next(), &mut prefijo)?,
                _ => {
                    return Err(format!("Opcion desconocida {}, gir hash-object [-t <type>] [-w] [--path=<file>] <file>", arg));
                }
            }
        }
        let ruta = format!("{}{}", prefijo, nombre_archivo);
        Ok(HashObject {
            logger,
            tipo_objeto,
            escribir,
            ruta,
        })
    }

    fn hashear_objeto(&self) -> Result<String, String> {
        let contenido = match fs::read_to_string(self.ruta.clone()) {
            Ok(contenido) => contenido,
            Err(_) => {
                return Err("No se pudo leer el archivo".to_string());
            }
        };
        let hash = self.hashear_contenido_objeto(&contenido);
        Ok(hash)
    }

    fn hashear_contenido_objeto(&self, contenido: &str) -> String {
        let header = format!("{} {}\0", self.tipo_objeto, contenido.len());
        let contenido_total = header + contenido;
        let mut hasher = Sha1::new();
        hasher.update(contenido_total);
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

    pub fn ejecutar(&self) -> Result<(), String> {
        let hash = self.hashear_objeto()?;
        println!("{}", hash);
        if self.escribir {
            let ruta = format!(".gir/objects/{}/{}", &hash[..2], &hash[2..]);
            let contenido = io::leer_a_string(&self.ruta.clone())?;

            io::escribir_bytes(&ruta, self.comprimir_contenido(contenido)?)?;
        }
        let mensaje = format!("Objeto gir hasheado en {}", self.ruta);
        self.logger.log(mensaje);
        Ok(())
    }
}

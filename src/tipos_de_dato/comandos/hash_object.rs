use crate::utilidades_de_compresion::comprimir_contenido;
use crate::{io, tipos_de_dato::logger::Logger};
use sha1::{Digest, Sha1};
use std::path::PathBuf;
use std::sync::Arc;

pub struct HashObject {
    pub logger: Arc<Logger>,
    pub escribir: bool,
    pub ubicacion_archivo: PathBuf,
}

impl HashObject {
    fn obtener_nombre_archivo(args: &mut Vec<String>) -> Result<PathBuf, String> {
        let nombre_string = args
            .pop()
            .ok_or_else(|| "No se especifico un archivo".to_string());
        Ok(PathBuf::from(nombre_string?))
    }

    pub fn from(args: &mut Vec<String>, logger: Arc<Logger>) -> Result<HashObject, String> {
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
            ubicacion_archivo: nombre_archivo,
            escribir,
        })
    }

    fn construir_contenido(&self) -> Result<String, String> {
        let contenido = io::leer_a_string(self.ubicacion_archivo.clone())?;
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

    pub fn ejecutar(&self) -> Result<String, String> {
        let contenido = self.construir_contenido()?;
        let hash = self.hashear_contenido_objeto(&contenido);

        if self.escribir {
            let ruta = format!(".gir/objects/{}/{}", &hash[..2], &hash[2..]);
            io::escribir_bytes(ruta, comprimir_contenido(contenido)?)?;
        }
        let mensaje = format!(
            "Objeto gir hasheado en {}",
            self.ubicacion_archivo.to_string_lossy()
        );
        self.logger.log(mensaje);
        Ok(hash)
    }
}

#[cfg(test)]
mod test {
    use std::{io::Read, path::PathBuf, sync::Arc};

    use flate2::read::ZlibDecoder;

    use crate::{
        io,
        tipos_de_dato::{comandos::hash_object::HashObject, logger::Logger},
    };

    #[test]
    fn test01_hash_object_de_un_blob_devuelve_el_hash_correcto() {
        let mut args = vec!["test_dir/objetos/archivo.txt".to_string()];
        let logger = Arc::new(Logger::new(PathBuf::from("tmp/hash_object_test01")).unwrap());
        let hash_object = HashObject::from(&mut args, logger).unwrap();
        let hash = hash_object.ejecutar().unwrap();
        assert_eq!(hash, "2b824e648965b94c6c6b3dd0702feb91f699ed62");
    }

    #[test]
    fn test02_hash_object_de_un_blob_con_opcion_w_devuelve_el_hash_correcto_y_lo_escribe() {
        let mut args = vec!["-w".to_string(), "test_dir/objetos/archivo.txt".to_string()];
        let logger = Arc::new(Logger::new(PathBuf::from("tmp/hash_object_test01")).unwrap());
        let hash_object = HashObject::from(&mut args, logger).unwrap();
        let hash = hash_object.ejecutar().unwrap();
        assert_eq!(hash, "2b824e648965b94c6c6b3dd0702feb91f699ed62");
        let contenido_leido =
            io::leer_bytes(".gir/objects/2b/824e648965b94c6c6b3dd0702feb91f699ed62").unwrap();
        let mut descompresor = ZlibDecoder::new(contenido_leido.as_slice());
        let mut contenido_descomprimido = String::new();
        descompresor
            .read_to_string(&mut contenido_descomprimido)
            .unwrap();
        assert_eq!(contenido_descomprimido, "blob 23\0contenido de un arxhivo");
    }
}

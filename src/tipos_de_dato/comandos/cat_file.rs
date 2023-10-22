use flate2::read::ZlibDecoder;

use crate::{
    io,
    tipos_de_dato::{logger::Logger, visualizaciones::Visualizaciones},
};
use std::{io::Read, rc::Rc};

pub struct CatFile {
    pub logger: Rc<Logger>,
    pub visualizacion: Visualizaciones,
    pub objeto: String,
}

impl CatFile {
    pub fn from(args: &mut Vec<String>, logger: Rc<Logger>) -> Result<CatFile, String> {
        let objeto = match args.pop() {
            Some(objeto) => objeto,
            None => {
                return Err(format!("No se asigno un objeto un objeto"));
            }
        };
        let segundo_argumento = match args.pop() {
            Some(segundo_argumento) => segundo_argumento,
            None => {
                return Err(format!(
                    "Se esperaba una opcion de visualizacion (-t | -s | -p)"
                ));
            }
        };
        let visualizacion = Visualizaciones::from(segundo_argumento)?;
        Ok(CatFile {
            logger,
            visualizacion,
            objeto,
        })
    }

    fn  descomprimir_objeto(&self) -> Result<String, String> {
        let ruta_objeto = format!("./.git/objects/{}/{}", &self.objeto[..2], &self.objeto[2..]);
        let contenido_leido = io::leer_bytes(&ruta_objeto)?;
        let mut descompresor = ZlibDecoder::new(contenido_leido.as_slice());
        let mut contenido_descomprimido = String::new();
        match descompresor.read_to_string(&mut contenido_descomprimido) {
            Ok(_) => {}
            Err(_) => {
                return Err(format!("No se pudo descomprimir el objeto {}", self.objeto));
            }
        };
        Ok(contenido_descomprimido)
    }

    fn obtener_objeto(&self) -> Result<(String, String), String> {
        let objeto = self.descomprimir_objeto()?;
        match objeto.split_once("\0") {
            Some((header, contenido)) => Ok((header.to_string(), contenido.to_string())),
            None => Err("Objeto invalido".to_string()),
        }
    }

    fn visualizar_contenido(&self) -> Result<String, String> {
        let (_, contenido) = self.obtener_objeto()?;
        println!("{}", contenido);
        Ok(format!(
            "Visualizacion del contenido del objeto: {}",
            self.objeto
        ))
    }

    fn visualizar_tamanio(&self) -> Result<String, String> {
        let (header, _) = self.obtener_objeto()?;
        let size = match header.split_once(" ") {
            Some((_, size)) => size,
            None => return Err(format!("Objeto invalido")),
        };

        println!("Tamaño del objeto: {}", size);
        Ok(format!("{}", size))
    }

    fn visualizar_tipo_objeto(&self) -> Result<String, String> {
        let (header, _) = self.obtener_objeto()?;
        let tipo_objeto = match header.split_once(" ") {
            Some((tipo, _)) => tipo,
            None => return Err(format!("Objeto invalido")),
        };
        println!("Tipo de objeto: {}", tipo_objeto);
        Ok(tipo_objeto.to_string())
    }

    pub fn ejecutar(&self) -> Result<String, String> {
        let mensaje = match self.visualizacion {
            Visualizaciones::TipoObjeto => self.visualizar_tipo_objeto()?,
            Visualizaciones::Tamanio => self.visualizar_tamanio()?,
            Visualizaciones::Contenido => self.visualizar_contenido()?,
        };
        self.logger.log(mensaje.clone());
        Ok(mensaje)
    }
}

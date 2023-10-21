use crate::{
    tipos_de_dato::{logger::Logger, visualizaciones::Visualizaciones, objeto::flag_es_un_objeto_}, utilidades_de_compresion::descomprimir_objeto,
};
use std::rc::Rc;

pub struct CatFile {
    pub logger: Rc<Logger>,
    pub visualizacion: Visualizaciones,
    pub hash_objeto: String,
}


fn obtener_contenido_objeto(hash: String) -> Result<(String, String), String> {
    let objeto = descomprimir_objeto(hash)?;
    match objeto.split_once('\0') {
        Some((header, contenido)) => Ok((header.to_string(), contenido.to_string())),
        None => Err("Objeto invalido".to_string()),
    }
}

pub fn conseguir_contenido(hash: String) -> Result<String, String> {
    let (_, contenido) = obtener_contenido_objeto(hash)?;
    Ok(format!("{}", contenido))
}

pub fn conseguir_tamanio(hash: String) -> Result<String, String> {
    let (header, _) = obtener_contenido_objeto(hash)?;
    let size = match header.split_once(' ') {
        Some((_, size)) => size,
        None => return Err("Objeto invalido".to_string()),
    };
    Ok(format!("{}", size))
}

pub fn conseguir_tipo_objeto(hash: String) -> Result<String, String> {
    let (header, _) = obtener_contenido_objeto(hash)?;
    let tipo_objeto = match header.split_once(' ') {
        Some((tipo, _)) => tipo,
        None => return Err("Objeto invalido".to_string()),
    };
    Ok(format!("{}", tipo_objeto))
}

impl CatFile {
    pub fn from(args: &mut Vec<String>, logger: Rc<Logger>) -> Result<CatFile, String> {
        let objeto = args.pop().ok_or_else(|| "No se especifico un objeto".to_string())?;
        let segundo_argumento = args.pop().ok_or_else(|| "No se especifico una opcion de visualizacion (-t | -s | -p)".to_string())?;
        let visualizacion = match flag_es_un_objeto_(&segundo_argumento) {
            true => Visualizaciones::from("-p".to_string())?,
            false => Visualizaciones::from(segundo_argumento)?,
        };
        Ok(CatFile {
            logger,
            visualizacion,
            hash_objeto: objeto,
        })
    }

    pub fn ejecutar(&self) -> Result<String, String> {
        let mensaje = match self.visualizacion {
            Visualizaciones::TipoObjeto => conseguir_tipo_objeto(self.hash_objeto.clone())?,
            Visualizaciones::Tamanio => conseguir_tamanio(self.hash_objeto.clone())?,
            Visualizaciones::Contenido => conseguir_contenido(self.hash_objeto.clone())?,
        };
        println!("{}", mensaje);
        self.logger.log(mensaje.clone());
        Ok(mensaje)
    }
}

#[cfg(test)]
mod test {
    use crate::{
        tipos_de_dato::{
            comandos::{cat_file::CatFile, hash_object::HashObject},
            logger::Logger,
            visualizaciones::Visualizaciones,
        },
        io,
    };
    use std::rc::Rc;

    #[test]
    fn test01_cat_file_blob_para_visualizar_muestra_el_contenido_correcto() {
        let logger = Rc::new(Logger::new().unwrap());
        let hash_object = HashObject::from(&mut vec!["-w".to_string(), "test_dir/objetos/archivo.txt".to_string()], logger.clone()).unwrap();
        let hash = hash_object.ejecutar().unwrap();
        let cat_file = CatFile {
            logger,
            visualizacion: Visualizaciones::Contenido,
            hash_objeto: hash.to_string(),
        };

        let contenido = cat_file.ejecutar().unwrap();
        let contenido_esperado = io::leer_a_string(&"test_dir/objetos/archivo.txt".to_string())
            .unwrap()
            .trim()
            .to_string();
        assert_eq!(contenido, contenido_esperado);
    }   

    #[test]
    fn test02_cat_file_blob_muestra_el_tamanio_correcto() {
        let logger = Rc::new(Logger::new().unwrap());
        let hash_object = HashObject::from(&mut vec!["-w".to_string(), "test_dir/objetos/archivo.txt".to_string()], logger.clone()).unwrap();
        let hash = hash_object.ejecutar().unwrap();
        let cat_file = CatFile {
            logger,
            visualizacion: Visualizaciones::Tamanio,
            hash_objeto: hash.to_string(),
        };
        let tamanio = cat_file.ejecutar().unwrap();
        let tamanio_esperado = io::leer_a_string(&"test_dir/objetos/archivo.txt".to_string())
            .unwrap()
            .trim()
            .len()
            .to_string();
        assert_eq!(tamanio, tamanio_esperado);
    }

    #[test]
    fn test03_cat_file_blob_muestra_el_tipo_de_objeto_correcto() {
        let logger = Rc::new(Logger::new().unwrap());
        let hash_object = HashObject::from(&mut vec!["-w".to_string(), "test_dir/objetos/archivo.txt".to_string()], logger.clone()).unwrap();
        let hash = hash_object.ejecutar().unwrap();
        let cat_file = CatFile {
            logger,
            visualizacion: Visualizaciones::TipoObjeto,
            hash_objeto: hash.to_string(),
        };
        let tipo_objeto = cat_file.ejecutar().unwrap();
        assert_eq!(tipo_objeto, "blob");
    }
}
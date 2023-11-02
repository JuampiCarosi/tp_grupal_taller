use crate::{
    tipos_de_dato::{
        logger::Logger, objeto::flag_es_un_objeto_, objetos::tree::Tree,
        visualizaciones::Visualizaciones,
    },
    utils::compresion::descomprimir_objeto,
};
use std::sync::Arc;

pub struct CatFile {
    pub logger: Arc<Logger>,
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

pub fn conseguir_tipo_objeto(header: String) -> Result<String, String> {
    let tipo_objeto = match header.split_once(' ') {
        Some((tipo, _)) => tipo,
        None => return Err("Objeto invalido".to_string()),
    };
    Ok(tipo_objeto.to_string())
}

pub fn conseguir_contenido_pretty(header: String, contenido: String) -> Result<String, String> {
    let tipo = conseguir_tipo_objeto(header)?;
    match tipo.as_str() {
        "blob" | "commit" => Ok(contenido),
        "tree" => {
            let mut pretty_print = String::new();
            let contenido_parseado = Tree::rearmar_contenido_descomprimido(contenido)?;
            let lineas = contenido_parseado.split('\n').collect::<Vec<&str>>();
            for linea in lineas {
                let atributos_objeto = linea.split(' ').collect::<Vec<&str>>();
                let mut modo = atributos_objeto[0].to_string();
                let tipo = match modo.as_str() {
                    "100644" => "blob".to_string(),
                    "40000" => "tree".to_string(),
                    _ => return Err("Objeto invalido".to_string()),
                };
                if modo == "40000" {
                    modo = "040000".to_string();
                }
                let nombre = atributos_objeto[1].to_string();
                let hash = atributos_objeto[2].to_string();
                let linea = format!("{} {} {}   {}\n", modo, tipo, hash, nombre);
                pretty_print.push_str(&linea);
            }
            Ok(pretty_print)
        }
        _ => Err("Objeto invalido".to_string()),
    }
}

pub fn conseguir_tamanio(header: String) -> Result<String, String> {
    let size = match header.split_once(' ') {
        Some((_, size)) => size,
        None => return Err("Objeto invalido".to_string()),
    };
    Ok(size.to_string())
}

impl CatFile {
    pub fn from(args: &mut Vec<String>, logger: Arc<Logger>) -> Result<CatFile, String> {
        let objeto = args
            .pop()
            .ok_or_else(|| "No se especifico un objeto".to_string())?;
        let segundo_argumento = args.pop().ok_or_else(|| {
            "No se especifico una opcion de visualizacion (-t | -s | -p)".to_string()
        })?;
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
        let (header, contenido) = obtener_contenido_objeto(self.hash_objeto.clone())?;
        let mensaje = match self.visualizacion {
            Visualizaciones::TipoObjeto => conseguir_tipo_objeto(header)?,
            Visualizaciones::Tamanio => conseguir_tamanio(header)?,
            Visualizaciones::Contenido => conseguir_contenido_pretty(header, contenido)?,
        };
        self.logger.log(mensaje.clone());
        Ok(mensaje)
    }
}

#[cfg(test)]
mod test {
    use crate::{
        io,
        tipos_de_dato::{
            comandos::{
                cat_file::{
                    conseguir_contenido_pretty, conseguir_tamanio, conseguir_tipo_objeto, CatFile,
                },
                hash_object::HashObject,
            },
            logger::Logger,
            visualizaciones::Visualizaciones,
        },
    };
    use std::{path::PathBuf, sync::Arc};

    #[test]
    fn test01_cat_file_blob_para_visualizar_muestra_el_contenido_correcto() {
        let logger = Arc::new(Logger::new(PathBuf::from("tmp/cat_file_test01")).unwrap());
        let hash_object = HashObject::from(
            &mut vec!["-w".to_string(), "test_dir/objetos/archivo.txt".to_string()],
            logger.clone(),
        )
        .unwrap();
        let hash = hash_object.ejecutar().unwrap();
        let cat_file = CatFile {
            logger,
            visualizacion: Visualizaciones::Contenido,
            hash_objeto: hash.to_string(),
        };

        let contenido = cat_file.ejecutar().unwrap();
        let contenido_esperado = io::leer_a_string("test_dir/objetos/archivo.txt")
            .unwrap()
            .trim()
            .to_string();
        assert_eq!(contenido, contenido_esperado);
    }

    #[test]
    fn test02_cat_file_blob_muestra_el_tamanio_correcto() {
        let logger = Arc::new(Logger::new(PathBuf::from("tmp/cat_file_test02")).unwrap());
        let hash_object = HashObject::from(
            &mut vec!["-w".to_string(), "test_dir/objetos/archivo.txt".to_string()],
            logger.clone(),
        )
        .unwrap();
        let hash = hash_object.ejecutar().unwrap();
        let cat_file = CatFile {
            logger,
            visualizacion: Visualizaciones::Tamanio,
            hash_objeto: hash.to_string(),
        };
        let tamanio = cat_file.ejecutar().unwrap();
        let tamanio_esperado = io::leer_a_string("test_dir/objetos/archivo.txt")
            .unwrap()
            .trim()
            .len()
            .to_string();
        assert_eq!(tamanio, tamanio_esperado);
    }

    #[test]
    fn test03_cat_file_blob_muestra_el_tipo_de_objeto_correcto() {
        let logger = Arc::new(Logger::new(PathBuf::from("tmp/cat_file_test03")).unwrap());
        let hash_object = HashObject::from(
            &mut vec!["-w".to_string(), "test_dir/objetos/archivo.txt".to_string()],
            logger.clone(),
        )
        .unwrap();
        let hash = hash_object.ejecutar().unwrap();
        let cat_file = CatFile {
            logger,
            visualizacion: Visualizaciones::TipoObjeto,
            hash_objeto: hash.to_string(),
        };
        let tipo_objeto = cat_file.ejecutar().unwrap();
        assert_eq!(tipo_objeto, "blob");
    }

    #[test]
    fn test03_pretty_print_tree_muestra_el_contenido_correcto() {
        let contenido = "40000 test_dir\0d1bd5884df89a9734e3b0a4e7721a4802d85cce8100644 test_file.txt\0678e12dc5c03a7cf6e9f64e688868962ab5d8b65".to_string();
        let pretty_print = conseguir_contenido_pretty("tree 109".to_string(), contenido).unwrap();
        assert_eq!(pretty_print, "040000 tree d1bd5884df89a9734e3b0a4e7721a4802d85cce8   test_dir\n100644 blob 678e12dc5c03a7cf6e9f64e688868962ab5d8b65   test_file.txt\n");
    }

    #[test]
    fn test04_conseguir_tipo_tree_muestra_el_tipo_de_objeto_correcto() {
        let tipo_objeto = conseguir_tipo_objeto("tree 109".to_string()).unwrap();
        assert_eq!(tipo_objeto, "tree");
    }

    #[test]
    fn test05_conseguir_tamanio_tree_muestra_el_tamanio_correcto() {
        let tamanio = conseguir_tamanio("tree 109".to_string()).unwrap();
        assert_eq!(tamanio, "109");
    }

    #[test]
    fn test06_pretty_print_commit_muestra_el_contenido_correcto() {
        let contenido = "tree c475b36be7b222b7ff1469b44b15cdc0f754ef44\n
        parent b557332b86888546cecbe81933cf22adb1f3fed1\n
        author aaaa <bbbb> 1698535611 -0300\n
        committer aaaa <bbbb> 1698535611 -0300'n"
            .to_string();
        let pretty_print =
            conseguir_contenido_pretty("commit 29".to_string(), contenido.clone()).unwrap();
        assert_eq!(pretty_print, contenido);
    }

    #[test]
    fn test07_conseguir_tipo_commit_muestra_el_tipo_de_objeto_correcto() {
        let tipo_objeto = conseguir_tipo_objeto("commit 109".to_string()).unwrap();
        assert_eq!(tipo_objeto, "commit");
    }

    #[test]
    fn test08_conseguir_tamanio_commit_muestra_el_tamanio_correcto() {
        let tamanio = conseguir_tamanio("commit 29".to_string()).unwrap();
        assert_eq!(tamanio, "29");
    }
}

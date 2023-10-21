use std::{fs, path::PathBuf, rc::Rc};

use crate::tipos_de_dato::{logger::Logger, objeto::Objeto};

use super::{hash_object::HashObject, update_index::UpdateIndex};

pub struct Add {
    logger: Rc<Logger>,
    objetos_con_ubicacion: Vec<(Objeto, PathBuf)>,
}

impl Add {
    fn obtener_hijos(path: String) -> Vec<(Objeto, PathBuf)> {
        let mut hijos: Vec<(Objeto, PathBuf)> = Vec::new();
        let mut iterador = std::fs::read_dir(path).unwrap();
        while let Some(entrada) = iterador.next() {
            let entrada = entrada.unwrap();
            let path = entrada.path();
            if path.file_name().unwrap() == ".DS_Store" {
                continue;
            }

            let objeto = Objeto::from_directorio(path.to_str().unwrap().to_string()).unwrap();
            hijos.push((objeto, path));
        }
        hijos
    }
    pub fn from(args: &mut Vec<String>, logger: Rc<Logger>) -> Result<Add, String> {
        let iterador = args.iter();
        let mut objetos: Vec<(Objeto, PathBuf)> = Vec::new();
        for arg in iterador {
            if fs::metadata(arg).unwrap().is_dir() {
                let mut hijos = Self::obtener_hijos(arg.to_string());
                objetos.append(&mut hijos);
            }
            let objeto = Objeto::from_directorio(arg.to_string())?;
            objetos.push((objeto, PathBuf::from(arg)));
        }

        Ok(Add {
            logger,
            objetos_con_ubicacion: objetos,
        })
    }

    pub fn execute(&self) -> Result<(), String> {
        self.logger.log("Ejecutando add".to_string());
        for (objeto, ubicacion) in &self.objetos_con_ubicacion {
            match objeto {
                Objeto::Blob(_) => {
                    HashObject {
                        logger: self.logger.clone(),
                        escribir: true,
                        nombre_archivo: ubicacion.to_str().unwrap().to_string(),
                    }
                    .ejecutar()?;

                    let directorios_padre = ubicacion.parent().unwrap();
                    for directorio in directorios_padre.ancestors() {
                        if directorio.to_str().unwrap() == "" {
                            break;
                        }
                        let tree =
                            Objeto::from_directorio(directorio.to_str().unwrap().to_string());
                        if let Ok(Objeto::Tree(tree)) = tree {
                            tree.escribir_en_base()?;
                        } else {
                            Err("No se pudo escribir el arbol")?;
                        }
                    }
                }

                Objeto::Tree(tree) => tree.escribir_en_base()?,
            }
        }

        for objeto in &self.objetos_con_ubicacion {
            let objeto = objeto.0.clone();
            UpdateIndex::from(self.logger.clone(), objeto)
                .unwrap()
                .ejecutar()?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use std::rc::Rc;

    use crate::{
        io,
        tipos_de_dato::{
            comandos::{add::Add, init::Init},
            logger::Logger,
        },
    };

    fn clear_gir() {
        let _ = std::fs::remove_dir_all(".gir");

        Init {
            logger: Rc::new(Logger::new().unwrap()),
            path: "./.gir".to_string(),
        }
        .ejecutar()
        .unwrap();
    }

    #[test]
    fn test01_add_agrega_un_archivo_al_index() {
        clear_gir();
        let logger = Rc::new(Logger::new().unwrap());
        let mut args = vec!["test_dir/objetos/archivo.txt".to_string()];
        let add = Add::from(&mut args, logger.clone()).unwrap();
        add.execute().unwrap();

        let contenido = io::leer_a_string(&".gir/index".to_string()).unwrap();
        assert!(contenido.contains("archivo.txt"));
    }

    #[test]
    fn test02_add_agrega_un_directorio_al_index() {
        clear_gir();

        let logger = Rc::new(Logger::new().unwrap());
        let mut args = vec!["test_dir/objetos".to_string()];
        let add = Add::from(&mut args, logger.clone()).unwrap();
        add.execute().unwrap();

        let contenido = io::leer_a_string(&".gir/index".to_string()).unwrap();
        assert!(contenido.contains("archivo.txt"));
        assert!(contenido.contains("objetos"));
    }

    #[test]
    fn test03_add_agrega_un_directorio_con_dos_carpetas_y_archivos() {
        let logger = Rc::new(Logger::new().unwrap());
        let mut args = vec!["test_dir/".to_string()];
        let add = Add::from(&mut args, logger.clone()).unwrap();
        add.execute().unwrap();

        let contenido = io::leer_a_string(&".gir/index".to_string()).unwrap();
        assert!(contenido.contains("archivo.txt"));
        assert!(contenido.contains("archivo copia.txt"));
        assert!(contenido.contains("test_dir"));
        assert!(contenido.contains("objetos"));
        assert!(contenido.contains("muchos_objetos"));
    }
}

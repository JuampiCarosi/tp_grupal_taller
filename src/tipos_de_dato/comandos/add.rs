use std::{collections::HashMap, fs, path::PathBuf, rc::Rc};

use crate::tipos_de_dato::{
    logger::Logger,
    objeto::{self, Objeto},
    objetos::tree::Tree,
};

use super::{hash_object::HashObject, update_index::UpdateIndex};

pub struct Add {
    logger: Rc<Logger>,
    objetos_con_ubicacion: HashMap<PathBuf, Objeto>,
}

impl Add {
    fn obtener_hijos(path: String) -> HashMap<PathBuf, Objeto> {
        let mut hijos: HashMap<PathBuf, Objeto> = HashMap::new();
        let mut iterador = std::fs::read_dir(path).unwrap();
        while let Some(entrada) = iterador.next() {
            let entrada = entrada.unwrap();
            let path = entrada.path();
            if path.file_name().unwrap() == ".DS_Store" || path.file_name().unwrap() == "" {
                continue;
            }

            if fs::metadata(path.clone()).unwrap().is_dir() {
                let hijos_anidados = Self::obtener_hijos(path.to_str().unwrap().to_string());
                hijos.extend(hijos_anidados);
            }

            let objeto = Objeto::from_directorio(path.to_str().unwrap().to_string()).unwrap();
            hijos.insert(path, objeto);
        }
        hijos
    }

    fn obtener_padres(
        objetos: &mut HashMap<PathBuf, Objeto>,
        ubicacion: &PathBuf,
        objeto: &Objeto,
    ) -> HashMap<PathBuf, Objeto> {
        let mut padres: HashMap<PathBuf, Objeto> = HashMap::new();
        let directorio_padre = ubicacion.parent().unwrap();

        if objetos.contains_key(&PathBuf::from(directorio_padre)) {
            let objeto_padre = objetos.get_mut(&PathBuf::from(directorio_padre)).unwrap();
            if let Objeto::Tree(ref mut tree) = objeto_padre {
                tree.agregar_hijo(objeto.clone());
                padres.insert(PathBuf::from(directorio_padre), Objeto::Tree(tree.clone()));
            }
        } else {
            let mut objetos: Vec<Objeto> = Vec::new();
            objetos.push(objeto.clone());
            let objeto_padre = Objeto::Tree(Tree {
                directorio: directorio_padre.to_str().unwrap().to_string(),
                objetos,
            });
            padres.insert(PathBuf::from(directorio_padre), objeto_padre);
        }

        padres
    }
    pub fn from(args: &mut Vec<String>, logger: Rc<Logger>) -> Result<Add, String> {
        let iterador = args.iter();
        let mut objetos: HashMap<PathBuf, Objeto> = HashMap::new();
        for arg in iterador.clone() {
            if fs::metadata(arg).unwrap().is_dir() {
                let hijos = Self::obtener_hijos(arg.to_string());
                objetos.extend(hijos);
            }

            let objeto = Objeto::from_directorio(arg.to_string())?;
            objetos.insert(PathBuf::from(arg), objeto);
        }

        comando
            .objetos_con_ubicacion
            .iter()
            .for_each(|(ubicacion, objeto)| {
                Self::obtener_padres(&mut objetos, ubicacion, &objeto);
            });

        let mut comando = Add {
            logger,
            objetos_con_ubicacion: objetos.clone(),
        };

        Ok(comando)
    }

    pub fn execute(&self) -> Result<(), String> {
        self.logger.log("Ejecutando add".to_string());
        for (ubicacion, objeto) in &self.objetos_con_ubicacion {
            match objeto {
                Objeto::Blob(_) => {
                    HashObject {
                        logger: self.logger.clone(),
                        escribir: true,
                        nombre_archivo: ubicacion.to_str().unwrap().to_string(),
                    }
                    .ejecutar()?;
                }

                Objeto::Tree(tree) => tree.escribir_en_base()?,
            }
        }

        for (_, objeto) in &self.objetos_con_ubicacion {
            println!("{:?}", objeto);
            UpdateIndex::from(self.logger.clone(), objeto.clone())
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
        assert!(contenido.contains("test_dir"));
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

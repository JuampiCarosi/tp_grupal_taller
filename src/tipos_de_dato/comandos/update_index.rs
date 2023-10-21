use std::{
    fs::{self, OpenOptions},
    path::{Path, PathBuf},
    rc::Rc,
};

use crate::tipos_de_dato::{logger::Logger, objeto::Objeto, objetos::blob};
use std::io::prelude::*;

pub struct UpdateIndex {
    logger: Rc<Logger>,
    ubicacion: PathBuf,
    index: Vec<Objeto>,
}

impl UpdateIndex {
    fn crear_index() {
        if Path::new("./.gir/index").exists() {
            return;
        }
        let _ = fs::File::create("./.gir/index");
    }

    fn leer_index() -> Result<Vec<Objeto>, String> {
        let file = match OpenOptions::new().read(true).open("./.gir/index") {
            Ok(file) => file,
            Err(_) => return Err("No se pudo abrir el archivo index".to_string()),
        };

        let mut objetos: Vec<Objeto> = Vec::new();

        for line in std::io::BufReader::new(file).lines() {
            if let Some(line) = line.as_ref().ok() {
                let objeto = Objeto::from_index(line.to_string())?;
                objetos.push(objeto);
            }
        }
        Ok(objetos)
    }

    pub fn from(logger: Rc<Logger>, ubicacion: PathBuf) -> Result<UpdateIndex, String> {
        Self::crear_index();
        let index = Self::leer_index()?;
        Ok(UpdateIndex {
            logger,
            ubicacion,
            index,
        })
    }

    fn obtener_objetos_raiz(&self) -> Vec<Objeto> {
        self.index
            .iter()
            .filter_map(|x| match x {
                Objeto::Tree(tree) => {
                    let directorio_split: Vec<&str> = tree.directorio.split("/").collect();
                    println!("{:?}", directorio_split);
                    return if directorio_split.len() == 1 {
                        Some(Objeto::Tree(tree.clone()))
                    } else {
                        None
                    };
                }
                Objeto::Blob(blob) => {
                    let directorio_split: Vec<&str> = blob
                        .ubicacion
                        .into_iter()
                        .map(|x| x.to_str().unwrap())
                        .collect();
                    println!("{:?}", directorio_split);

                    return if directorio_split.len() == 1 {
                        Some(Objeto::Blob(blob.clone()))
                    } else {
                        None
                    };
                }
            })
            .collect()
    }

    fn escribir_objetos(&self) -> Result<(), String> {
        let mut file = match OpenOptions::new().write(true).open("./.gir/index") {
            Ok(file) => file,
            Err(_) => return Err("No se pudo escribir el archivo index".to_string()),
        };

        let objetos_raiz = self.obtener_objetos_raiz();

        for objeto in self.index.iter() {
            let line = match objeto {
                Objeto::Blob(blob) => format!("{blob}"),
                Objeto::Tree(tree) => format!("{tree}"),
            };

            let _ = file.write_all(b"");
            let _ = file.write_all(line.as_bytes());
        }
        Ok(())
    }

    pub fn ejecutar(&mut self) -> Result<(), String> {
        self.logger.log("Ejecutando update-index".to_string());

        let ubicacion_string = self
            .ubicacion
            .to_str()
            .ok_or_else(|| format!("ubicacion para update-index invalida {:?}", self.ubicacion))?
            .to_string();

        let nuevo_objeto = Objeto::from_directorio(ubicacion_string.clone())?;

        let indice = self.index.iter().position(|x| match x {
            Objeto::Blob(blob) => blob.ubicacion == PathBuf::from(&ubicacion_string),

            Objeto::Tree(tree) => tree.directorio == ubicacion_string,
        });

        let trees_a_actualizar = &mut self.index.iter().filter_map(|x| {
            if let Objeto::Tree(tree) = x {
                let directorio_tree = PathBuf::from(&tree.directorio);
                if self.ubicacion.starts_with(directorio_tree) {
                    return Some(Objeto::Tree(tree.clone()));
                }
            }
            None
        });

        for tree in trees_a_actualizar {
            if let Objeto::Tree(mut tree) = tree {
                tree.actualizar_hijos(nuevo_objeto.obtener_hash());
            }
        }

        if let Some(i) = indice {
            let _ = std::mem::replace(&mut self.index[i], nuevo_objeto);
        } else {
            self.index.push(nuevo_objeto);
        }

        self.escribir_objetos()?;
        Ok(())
    }
}

#[cfg(test)]

mod test {
    use std::{io::Write, path::PathBuf, rc::Rc};

    use crate::{
        io,
        tipos_de_dato::{comandos::update_index::UpdateIndex, logger::Logger, objeto::Objeto},
    };

    fn create_test_file() {
        let mut file = std::fs::File::create("test_file.txt").unwrap();
        let _ = file.write_all(b"test file");
    }

    fn modify_test_file() {
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .open("test_file.txt")
            .unwrap();
        let _ = file.write_all(b"test file modified");
    }

    fn clear_index() {
        let _ = std::fs::remove_file("./.gir/index");
    }

    #[test]
    fn test01_archivo_vacio_se_llena_con_objeto_agregado() {
        clear_index();
        create_test_file();
        let logger = Rc::new(Logger::new().unwrap());
        let ubicacion = PathBuf::from("test_file.txt");
        let mut update_index = UpdateIndex::from(logger, ubicacion).unwrap();

        update_index.ejecutar().unwrap();

        let index = update_index.index;

        assert_eq!(index.len(), 1);

        let file = io::leer_a_string(&"./.gir/index".to_string()).unwrap();
        assert_eq!(
            file,
            "100644 bdf08de0f3095da5030fecd9bafc0b00c1aced7c test_file.txt\n"
        );
    }

    #[test]
    fn test02_archivo_con_objeto_actualiza_el_objeto() {
        modify_test_file();
        let logger = Rc::new(Logger::new().unwrap());
        let ubicacion = PathBuf::from("test_file.txt");
        let mut update_index = UpdateIndex::from(logger, ubicacion).unwrap();

        update_index.ejecutar().unwrap();

        let index = update_index.index;

        assert_eq!(index.len(), 1);

        let objeto = &index[0];
        if let Objeto::Blob(blob) = objeto {
            assert_eq!(blob.nombre, "test_file.txt");
            assert_eq!(blob.hash, "678e12dc5c03a7cf6e9f64e688868962ab5d8b65");
        }

        let file = io::leer_a_string(&"./.gir/index".to_string()).unwrap();
        assert_eq!(
            file,
            "100644 678e12dc5c03a7cf6e9f64e688868962ab5d8b65 test_file.txt\n"
        );
    }

    #[test]
    fn test03_archivo_con_objetos_agrega_nuevos_objetos() {
        let _ = std::fs::remove_file("./.gir/index");
        let logger = Rc::new(Logger::new().unwrap());
        let ubicacion = PathBuf::from("test_file.txt");

        let mut update_index = UpdateIndex::from(logger.clone(), ubicacion).unwrap();
        update_index.ejecutar().unwrap();

        let ubicacion = PathBuf::from("test_dir/objetos/archivo.txt");

        let mut update_index = UpdateIndex::from(logger.clone(), ubicacion).unwrap();
        update_index.ejecutar().unwrap();

        let index = update_index.index;

        assert_eq!(index.len(), 2);

        let objeto = &index[0];

        if let Objeto::Blob(blob) = objeto {
            assert_eq!(blob.nombre, "test_file.txt");
            assert_eq!(blob.hash, "678e12dc5c03a7cf6e9f64e688868962ab5d8b65");
        }

        let objeto = &index[1];

        if let Objeto::Blob(blob) = objeto {
            assert_eq!(blob.nombre, "archivo.txt");
            assert_eq!(blob.hash, "2b824e648965b94c6c6b3dd0702feb91f699ed62");
        }

        let file = io::leer_a_string(&"./.gir/index".to_string()).unwrap();

        assert_eq!(
            file,
            "100644 678e12dc5c03a7cf6e9f64e688868962ab5d8b65 test_file.txt\n100644 2b824e648965b94c6c6b3dd0702feb91f699ed62 archivo.txt\n"
        );
    }

    #[test]
    fn test04_agregar_un_directorio_al_index() {
        clear_index();
        let logger = Rc::new(Logger::new().unwrap());

        let path = PathBuf::from("test_dir/objetos");
        let mut update_index = UpdateIndex::from(logger.clone(), path).unwrap();
        update_index.ejecutar().unwrap();

        let file = io::leer_a_string(&"./.gir/index".to_string()).unwrap();

        assert_eq!(
            file,
            "40000 bf902127ac66b999327fba07a9f4b7a50b87922a objetos\n"
        );
    }

    #[test]
    fn test05_agregar_un_objeto_en_un_directorio() {
        clear_index();

        let logger = Rc::new(Logger::new().unwrap());

        let path = PathBuf::from("test_dir/objetos/archivo.txt");
        let mut update_index = UpdateIndex::from(logger.clone(), path).unwrap();
        update_index.ejecutar().unwrap();

        let file = io::leer_a_string(&"./.gir/index".to_string()).unwrap();

        assert_eq!(
            file,
            "40000 bf902127ac66b999327fba07a9f4b7a50b87922a test_dir\n"
        );
    }

    #[test]
    fn test06_editar_hijo_actualiza_padre() {
        clear_index();

        let logger = Rc::new(Logger::new().unwrap());

        let tree_path = PathBuf::from("test_dir/muchos_objetos/archivo.txt");
        let mut update_index = UpdateIndex::from(logger.clone(), tree_path).unwrap();
        update_index.ejecutar().unwrap();

        let blob_path = PathBuf::from("test_dir/muchos_objetos/archivo copy.txt");

        let mut update_index = UpdateIndex::from(logger.clone(), blob_path).unwrap();
        update_index.ejecutar().unwrap();

        let file = io::leer_a_string(&"./.gir/index".to_string()).unwrap();

        assert_eq!(
            file,
            "100644 534b4ac42126f12 Readme.md\n100644 534b4ac42126f13 Cargo.toml\n40000 bf902127ac66b999327fba07a9f4b7a50b87922a objetos\n"
        );
    }
}

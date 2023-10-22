use std::{
    collections::HashMap,
    fs::{self, OpenOptions},
    path::{Path, PathBuf},
    rc::Rc,
};

use crate::tipos_de_dato::{comandos::hash_object::HashObject, logger::Logger, objeto::Objeto};
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

    fn generar_objetos_raiz(&self) -> Result<Vec<Objeto>, String> {
        let mut objetos: HashMap<String, Objeto> = HashMap::new();
        let mut directorios_raiz: HashMap<String, ()> = HashMap::new();
        let mut directorios_a_tener_en_cuenta: Vec<String> = Vec::new();
        self.index.iter().for_each(|x| match x {
            Objeto::Tree(tree) => directorios_a_tener_en_cuenta.extend(tree.obtener_paths_hijos()),
            Objeto::Blob(blob) => {
                directorios_a_tener_en_cuenta.push(blob.ubicacion.to_str().unwrap().to_string())
            }
        });

        for objeto in self.index.iter() {
            match objeto {
                Objeto::Blob(blob) => {
                    let directorio_split: Vec<&str> = blob
                        .ubicacion
                        .into_iter()
                        .map(|x| x.to_str().unwrap())
                        .collect();

                    if directorio_split.len() < 1 {
                        Err(format!("Directorio invalido {:?}", directorio_split))?;
                        self.logger.log(format!("Directorio invalido al intentar obtener los directorios raiz de cada objeto {:?}, la longitud del directorio es menor a 1", directorio_split));
                    }
                    directorios_raiz.insert(directorio_split[0].to_string(), ());
                }
                Objeto::Tree(tree) => {
                    let directorio_split: Vec<&str> = tree.directorio.split("/").collect();
                    if directorio_split.len() < 1 {
                        Err(format!("Directorio invalido {:?}", directorio_split))?;
                        self.logger.log(format!("Directorio invalido al intentar obtener los directorios raiz de cada objeto {:?}, la longitud del directorio es menor a 1", directorio_split));
                    }
                    directorios_raiz.insert(directorio_split[0].to_string(), ());
                }
            }
        }

        for directorio in directorios_raiz.keys() {
            if objetos.contains_key(directorio) {
                continue;
            }

            let objeto_conteniendo_al_blob = Objeto::from_directorio_con_hijos_especificados(
                directorio.clone(),
                &directorios_a_tener_en_cuenta,
            )?;

            objetos.insert(directorio.clone(), objeto_conteniendo_al_blob);
        }

        let mut vector: Vec<Objeto> = objetos.values().cloned().collect();
        vector.sort_by_key(|x| match x {
            Objeto::Blob(blob) => blob.ubicacion.clone(),
            Objeto::Tree(tree) => PathBuf::from(&tree.directorio),
        });
        Ok(vector)
    }

    fn escribir_objetos(&self) -> Result<(), String> {
        let mut file = match OpenOptions::new().write(true).open("./.gir/index") {
            Ok(file) => file,
            Err(_) => return Err("No se pudo escribir el archivo index".to_string()),
        };

        for objeto in self.generar_objetos_raiz()? {
            println!("{:?}", objeto);
            let line = match objeto {
                Objeto::Blob(blob) => {
                    HashObject {
                        logger: self.logger.clone(),
                        escribir: true,
                        nombre_archivo: blob.ubicacion.to_str().unwrap().to_string(),
                    }
                    .ejecutar()?;
                    format!("{blob}")
                }
                Objeto::Tree(tree) => {
                    tree.escribir_en_base()?;
                    format!("{tree}")
                }
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
    fn test03_agregar_un_objeto_en_un_directorio() {
        clear_index();

        let logger = Rc::new(Logger::new().unwrap());

        let path = PathBuf::from("test_dir/objetos/archivo.txt");
        let mut update_index = UpdateIndex::from(logger.clone(), path).unwrap();
        update_index.ejecutar().unwrap();

        let file = io::leer_a_string(&"./.gir/index".to_string()).unwrap();

        assert_eq!(
            file,
            "40000 59fd3de622aa800bc7e0684773ce2dc40d876377 test_dir\n"
        );
    }

    #[test]
    fn test04_archivo_con_objetos_agrega_nuevos_objetos() {
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
            "40000 59fd3de622aa800bc7e0684773ce2dc40d876377 test_dir\n100644 678e12dc5c03a7cf6e9f64e688868962ab5d8b65 test_file.txt\n"
        );
    }

    #[test]
    fn test05_agregar_un_directorio_al_index() {
        clear_index();
        let logger = Rc::new(Logger::new().unwrap());

        let path = PathBuf::from("test_dir/muchos_objetos");
        let mut update_index = UpdateIndex::from(logger.clone(), path).unwrap();
        update_index.ejecutar().unwrap();

        let file = io::leer_a_string(&"./.gir/index".to_string()).unwrap();

        assert_eq!(
            file,
            "40000 9ce160c141eb52a2479bd734e54e0a64a49c5d76 test_dir\n"
        );
    }

    #[test]
    fn test06_editar_hijo_actualiza_padre() {
        clear_index();

        let logger = Rc::new(Logger::new().unwrap());

        let archivo_1 = PathBuf::from("test_dir/muchos_objetos/archivo.txt");
        let mut update_index = UpdateIndex::from(logger.clone(), archivo_1).unwrap();
        update_index.ejecutar().unwrap();

        let file = io::leer_a_string(&"./.gir/index".to_string()).unwrap();

        assert_eq!(
            file,
            "40000 dff341b050347f57726eb70b97addee11ea73194 test_dir\n"
        );

        let archivo_2 = PathBuf::from("test_dir/muchos_objetos/archivo copy.txt");

        let mut update_index = UpdateIndex::from(logger.clone(), archivo_2).unwrap();
        update_index.ejecutar().unwrap();

        let file = io::leer_a_string(&"./.gir/index".to_string()).unwrap();

        assert_eq!(
            file,
            "40000 9ce160c141eb52a2479bd734e54e0a64a49c5d76 test_dir\n"
        );
    }
}

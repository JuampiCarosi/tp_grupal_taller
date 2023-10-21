use std::{
    fs::{self, OpenOptions},
    path::{Path, PathBuf},
    rc::Rc,
};

use crate::tipos_de_dato::{logger::Logger, objeto::Objeto};
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

    fn escribir_objetos(&self) -> Result<(), String> {
        let mut file = match OpenOptions::new().write(true).open("./.gir/index") {
            Ok(file) => file,
            Err(_) => return Err("No se pudo escribir el archivo index".to_string()),
        };
        for objeto in &self.index {
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

    #[test]
    fn test01_archivo_vacio_se_llena_con_objeto_agregado() {
        let _ = std::fs::remove_file("./.gir/index");
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

    // #[test]
    // fn test03_archivo_con_objetos_agrega_nuevos_objetos() {
    //     let logger = Rc::new(Logger::new().unwrap());
    //     let objeto = Objeto::Blob(Blob {
    //         nombre: "Readme.md".to_string(),
    //         hash: "534b4ac42126f12".to_string(),
    //     });

    //     let mut update_index = UpdateIndex::from(logger.clone(), objeto).unwrap();

    //     update_index.ejecutar().unwrap();

    //     let objeto = Objeto::Blob(Blob {
    //         nombre: "Cargo.toml".to_string(),
    //         hash: "534b4ac42126f13".to_string(),
    //     });

    //     let mut update_index = UpdateIndex::from(logger.clone(), objeto).unwrap();

    //     update_index.ejecutar().unwrap();

    //     let index = update_index.index;

    //     assert_eq!(index.len(), 2);

    //     let objeto = &index[0];

    //     if let Objeto::Blob(blob) = objeto {
    //         assert_eq!(blob.nombre, "Readme.md");
    //         assert_eq!(blob.hash, "534b4ac42126f12");
    //     }

    //     let objeto = &index[1];

    //     if let Objeto::Blob(blob) = objeto {
    //         assert_eq!(blob.nombre, "Cargo.toml");
    //         assert_eq!(blob.hash, "534b4ac42126f13");
    //     }

    //     let file = io::leer_a_string(&"./.gir/index".to_string()).unwrap();

    //     assert_eq!(
    //         file,
    //         "100644 534b4ac42126f12 Readme.md\n100644 534b4ac42126f13 Cargo.toml\n"
    //     );
    // }

    // #[test]
    // fn test04_agregar_un_directorio_al_index() {
    //     let logger = Rc::new(Logger::new().unwrap());

    //     let tree = Objeto::from_directorio("test_dir/objetos".to_string()).unwrap();
    //     let mut update_index = UpdateIndex::from(logger.clone(), tree).unwrap();
    //     update_index.ejecutar().unwrap();

    //     let file = io::leer_a_string(&"./.gir/index".to_string()).unwrap();

    //     assert_eq!(
    //         file,
    //         "100644 534b4ac42126f12 Readme.md\n100644 534b4ac42126f13 Cargo.toml\n40000 bf902127ac66b999327fba07a9f4b7a50b87922a objetos\n"
    //     );
    // }

    // #[test]
    // fn test05_editar_hijo_actualiza_padre() {
    //     // let logger = Rc::new(Logger::new().unwrap());

    //     // let tree = Objeto::from_directorio("test_dir/objetos".to_string()).unwrap();
    //     // let mut update_index = UpdateIndex::from(logger.clone(), tree).unwrap();
    //     // update_index.ejecutar().unwrap();

    //     // let objeto = Objeto::Blob(Blob {
    //     //     nombre: "archivo.txt".to_string(),
    //     //     hash: "534b4ac42126f13".to_string(),
    //     // });

    //     // let mut update_index = UpdateIndex::from(logger.clone(), objeto).unwrap();
    //     // update_index.ejecutar().unwrap();

    //     // let file = io::leer_a_string(&"./.gir/index".to_string()).unwrap();

    //     // assert_eq!(
    //     //     file,
    //     //     "100644 534b4ac42126f12 Readme.md\n100644 534b4ac42126f13 Cargo.toml\n40000 bf902127ac66b999327fba07a9f4b7a50b87922a objetos\n"
    //     // );
    // }
}

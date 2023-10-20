use std::{
    fs::{self, OpenOptions},
    ops::Index,
    path::Path,
    rc::Rc,
};

use crate::tipos_de_dato::{logger::Logger, objeto::Objeto};
use std::io::prelude::*;

pub struct UpdateIndex {
    logger: Rc<Logger>,
    objeto: Objeto,
    index: Vec<Objeto>,
}

impl UpdateIndex {
    fn crear_index() {
        if Path::new("./.gir/index").exists() {
            return;
        }
        let _ = fs::File::create("./.gir/index");
    }

    fn leer_index() -> Vec<Objeto> {
        let file = OpenOptions::new().read(true).open("./.gir/index").unwrap();

        let mut objetos: Vec<Objeto> = Vec::new();

        for line in std::io::BufReader::new(file).lines() {
            let objeto = Objeto::from_index(line.unwrap()).unwrap();
            objetos.push(objeto);
        }
        objetos
    }

    pub fn from(logger: Rc<Logger>, objeto: Objeto) -> Result<UpdateIndex, String> {
        Self::crear_index();
        let index = Self::leer_index();
        Ok(UpdateIndex {
            logger,
            objeto,
            index,
        })
    }

    fn escribir_objetos(&self) {
        let mut file = OpenOptions::new().write(true).open("./.gir/index").unwrap();

        for objeto in &self.index {
            let line = match objeto {
                Objeto::Blob(blob) => format!("{blob}"),
                Objeto::Tree(tree) => format!("{tree}"),
            };

            file.write_all(line.as_bytes()).unwrap();
        }
    }

    pub fn ejecutar(&mut self) -> Result<(), String> {
        self.logger.log("Ejecutando update-index".to_string());

        let indice = self.index.iter().position(|x| {
            if let Objeto::Blob(blob) = x {
                if let Objeto::Blob(blob2) = &self.objeto {
                    return blob.nombre == blob2.nombre;
                }
            }
            false
        });

        let trees_a_actualizar = &mut self.index.iter().filter_map(|x| {
            if let Objeto::Tree(tree) = x {
                if tree.contiene_hijo(self.objeto.obtener_hash()) {
                    return Some(x.clone());
                };
            }
            None
        });

        for tree in trees_a_actualizar {
            if let Objeto::Tree(mut tree) = tree {
                tree.actualizar_hijos(self.objeto.obtener_hash());
            }
        }

        if let Some(i) = indice {
            let _ = std::mem::replace(&mut self.index[i], self.objeto.clone());
        } else {
            self.index.push(self.objeto.clone());
        }

        self.escribir_objetos();
        Ok(())
    }
}

#[cfg(test)]

mod test {
    use std::rc::Rc;

    use crate::{
        io,
        tipos_de_dato::{
            comandos::update_index::UpdateIndex, logger::Logger, objeto::Objeto,
            objetos::blob::Blob,
        },
    };

    #[test]
    fn test01_archivo_vacio_se_llena_con_objeto_agregado() {
        let logger = Rc::new(Logger::new().unwrap());
        let objeto = Objeto::Blob(Blob {
            nombre: "Readme.md".to_string(),
            hash: "534b4ac42126f12".to_string(),
        });

        let mut update_index = UpdateIndex::from(logger, objeto).unwrap();

        update_index.ejecutar().unwrap();

        let index = update_index.index;

        assert_eq!(index.len(), 1);

        let objeto = &index[0];

        if let Objeto::Blob(blob) = objeto {
            assert_eq!(blob.nombre, "Readme.md");
            assert_eq!(blob.hash, "534b4ac42126f12");
        }

        let file = io::leer_a_string(&"./.gir/index".to_string()).unwrap();

        assert_eq!(file, "100644 534b4ac42126f12 Readme.md\n");
    }

    #[test]
    fn test02_archivo_con_objeto_actualiza_el_objeto() {
        let logger = Rc::new(Logger::new().unwrap());
        let objeto = Objeto::Blob(Blob {
            nombre: "Readme.md".to_string(),
            hash: "534b4ac42126f12".to_string(),
        });

        let mut update_index = UpdateIndex::from(logger.clone(), objeto).unwrap();

        update_index.ejecutar().unwrap();

        let objeto = Objeto::Blob(Blob {
            nombre: "Readme.md".to_string(),
            hash: "534b4ac42126f13".to_string(),
        });

        let mut update_index = UpdateIndex::from(logger.clone(), objeto).unwrap();

        update_index.ejecutar().unwrap();

        let index = update_index.index;

        assert_eq!(index.len(), 1);

        let objeto = &index[0];

        if let Objeto::Blob(blob) = objeto {
            assert_eq!(blob.nombre, "Readme.md");
            assert_eq!(blob.hash, "534b4ac42126f13");
        }

        let file = io::leer_a_string(&"./.gir/index".to_string()).unwrap();

        assert_eq!(file, "100644 534b4ac42126f13 Readme.md\n");
    }

    #[test]
    fn test03_archivo_con_objetos_agrega_nuevos_objetos() {
        let logger = Rc::new(Logger::new().unwrap());
        let objeto = Objeto::Blob(Blob {
            nombre: "Readme.md".to_string(),
            hash: "534b4ac42126f12".to_string(),
        });

        let mut update_index = UpdateIndex::from(logger.clone(), objeto).unwrap();

        update_index.ejecutar().unwrap();

        let objeto = Objeto::Blob(Blob {
            nombre: "Cargo.toml".to_string(),
            hash: "534b4ac42126f13".to_string(),
        });

        let mut update_index = UpdateIndex::from(logger.clone(), objeto).unwrap();

        update_index.ejecutar().unwrap();

        let index = update_index.index;

        assert_eq!(index.len(), 2);

        let objeto = &index[0];

        if let Objeto::Blob(blob) = objeto {
            assert_eq!(blob.nombre, "Readme.md");
            assert_eq!(blob.hash, "534b4ac42126f12");
        }

        let objeto = &index[1];

        if let Objeto::Blob(blob) = objeto {
            assert_eq!(blob.nombre, "Cargo.toml");
            assert_eq!(blob.hash, "534b4ac42126f13");
        }

        let file = io::leer_a_string(&"./.gir/index".to_string()).unwrap();

        assert_eq!(
            file,
            "100644 534b4ac42126f12 Readme.md\n100644 534b4ac42126f13 Cargo.toml\n"
        );
    }

    #[test]
    fn test04_agregar_un_directorio_al_index() {
        let logger = Rc::new(Logger::new().unwrap());

        let tree = Objeto::from_directorio("test_dir/objetos".to_string()).unwrap();
        let mut update_index = UpdateIndex::from(logger.clone(), tree).unwrap();
        update_index.ejecutar().unwrap();

        let file = io::leer_a_string(&"./.gir/index".to_string()).unwrap();

        assert_eq!(
            file,
            "100644 534b4ac42126f12 Readme.md\n100644 534b4ac42126f13 Cargo.toml\n40000 bf902127ac66b999327fba07a9f4b7a50b87922a objetos\n"
        );
    }

    // fn test05_editar_hijo_actualiza_padre() {
    //     let logger = Rc::new(Logger::new().unwrap());

    //     let tree = Objeto::from_directorio("test_dir/objetos".to_string()).unwrap();
    //     let mut update_index = UpdateIndex::from(logger.clone(), tree).unwrap();
    //     update_index.ejecutar().unwrap();

    //     let objeto = Objeto::Blob(Blob {
    //         nombre: "archivo.txt".to_string(),
    //         hash: "534b4ac42126f13".to_string(),
    //     });

    //     let mut update_index = UpdateIndex::from(logger.clone(), objeto).unwrap();
    //     update_index.ejecutar().unwrap();

    //     let file = io::leer_a_string(&"./.gir/index".to_string()).unwrap();

    //     assert_eq!(
    //         file,
    //         "100644 534b4ac42126f12 Readme.md\n100644 534b4ac42126f13 Cargo.toml\n40000 bf902127ac66b999327fba07a9f4b7a50b87922a objetos\n"
    //     );
    // }
}

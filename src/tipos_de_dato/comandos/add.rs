use std::{
    collections::HashSet,
    fs::{self, OpenOptions},
    path::{Path, PathBuf},
    rc::Rc,
};

use crate::{
    tipos_de_dato::{comandos::hash_object::HashObject, logger::Logger, objeto::Objeto},
    utilidades_path_buf::obtener_directorio_raiz,
};
use std::io::prelude::*;

pub struct Add {
    logger: Rc<Logger>,
    ubicaciones: Vec<PathBuf>,
    index: Vec<Objeto>,
}

impl Add {
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
            if let Ok(line) = line.as_ref() {
                let objeto = Objeto::from_index(line.to_string())?;
                objetos.push(objeto);
            }
        }
        Ok(objetos)
    }

    pub fn from(args: Vec<String>, logger: Rc<Logger>) -> Result<Add, String> {
        Self::crear_index();
        let index = Self::leer_index()?;
        let ubicaciones = args.iter().map(PathBuf::from).collect::<Vec<PathBuf>>();

        Ok(Add {
            logger,
            ubicaciones,
            index,
        })
    }

    fn generar_objetos_raiz(&self) -> Result<Vec<Objeto>, String> {
        let mut objetos: Vec<Objeto> = Vec::new();
        let mut directorios_raiz: HashSet<PathBuf> = HashSet::new();
        let mut directorios_a_tener_en_cuenta: Vec<PathBuf> = Vec::new();
        self.index.iter().for_each(|x| match x {
            Objeto::Tree(tree) => directorios_a_tener_en_cuenta.extend(tree.obtener_paths_hijos()),
            Objeto::Blob(blob) => directorios_a_tener_en_cuenta.push(blob.ubicacion.clone()),
        });

        for objeto in self.index.iter() {
            match objeto {
                Objeto::Blob(blob) => {
                    let padre = obtener_directorio_raiz(&blob.ubicacion)?;
                    directorios_raiz.insert(PathBuf::from(padre));
                }
                Objeto::Tree(tree) => {
                    let padre = obtener_directorio_raiz(&tree.directorio)?;
                    directorios_raiz.insert(PathBuf::from(padre));
                }
            }
        }

        for directorio in directorios_raiz {
            let objeto_conteniendo_al_blob =
                Objeto::from_directorio(directorio.clone(), Some(&directorios_a_tener_en_cuenta))?;

            objetos.push(objeto_conteniendo_al_blob);
        }
        println!("OBJETO {:?}", directorios_a_tener_en_cuenta);

        objetos.sort_by_key(|x| match x {
            Objeto::Blob(blob) => blob.ubicacion.clone(),
            Objeto::Tree(tree) => PathBuf::from(&tree.directorio),
        });
        Ok(objetos)
    }

    fn escribir_objetos(&self) -> Result<(), String> {
        let mut file = match OpenOptions::new().write(true).open("./.gir/index") {
            Ok(file) => file,
            Err(_) => return Err("No se pudo escribir el archivo index".to_string()),
        };

        for objeto in self.generar_objetos_raiz()? {
            let line = match objeto {
                Objeto::Blob(blob) => {
                    HashObject {
                        logger: self.logger.clone(),
                        escribir: true,
                        ubicacion_archivo: blob.ubicacion.clone(),
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

    pub fn ejecutar(&mut self) -> Result<String, String> {
        self.logger.log("Ejecutando update-index".to_string());

        for ubicacion in self.ubicaciones.clone() {
            let nuevo_objeto = Objeto::from_directorio(ubicacion.clone(), None)?;

            let indice = self.index.iter().position(|x| match x {
                Objeto::Blob(blob) => blob.ubicacion == ubicacion,
                Objeto::Tree(tree) => tree.directorio == ubicacion,
            });

            let trees_a_actualizar = &mut self.index.iter().filter_map(|x| {
                if let Objeto::Tree(tree) = x {
                    let directorio_tree = PathBuf::from(&tree.directorio);
                    if ubicacion.starts_with(directorio_tree) {
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
        }

        self.escribir_objetos()?;
        Ok("".to_string())
    }
}

#[cfg(test)]

mod test {
    use std::{io::Write, path::PathBuf, rc::Rc};

    use crate::{
        io,
        tipos_de_dato::{comandos::add::Add, logger::Logger, objeto::Objeto},
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
        let logger = Rc::new(Logger::new(PathBuf::from("tmp/add_test01")).unwrap());
        let ubicacion = "test_file.txt".to_string();
        let mut add = Add::from(vec![ubicacion], logger).unwrap();

        add.ejecutar().unwrap();

        assert_eq!(add.index.len(), 1);

        let file = io::leer_a_string("./.gir/index").unwrap();
        assert_eq!(
            file,
            "100644 bdf08de0f3095da5030fecd9bafc0b00c1aced7c test_file.txt\n"
        );
    }

    #[test]
    fn test02_archivo_con_objeto_actualiza_el_objeto() {
        modify_test_file();
        let logger = Rc::new(Logger::new(PathBuf::from("tmp/add_test02")).unwrap());
        let ubicacion = "test_file.txt".to_string();
        let mut add = Add::from(vec![ubicacion], logger).unwrap();

        add.ejecutar().unwrap();

        assert_eq!(add.index.len(), 1);

        let objeto = &add.index[0];
        if let Objeto::Blob(blob) = objeto {
            assert_eq!(blob.nombre, "test_file.txt");
            assert_eq!(blob.hash, "678e12dc5c03a7cf6e9f64e688868962ab5d8b65");
        }

        let file = io::leer_a_string("./.gir/index").unwrap();
        assert_eq!(
            file,
            "100644 678e12dc5c03a7cf6e9f64e688868962ab5d8b65 test_file.txt\n"
        );
    }

    #[test]
    fn test03_agregar_un_objeto_en_un_directorio() {
        clear_index();

        let logger = Rc::new(Logger::new(PathBuf::from("tmp/add_test03")).unwrap());

        let path = "test_dir/objetos/archivo.txt".to_string();
        let mut add = Add::from(vec![path], logger.clone()).unwrap();
        add.ejecutar().unwrap();

        let file = io::leer_a_string("./.gir/index").unwrap();

        assert_eq!(
            file,
            "40000 59fd3de622aa800bc7e0684773ce2dc40d876377 test_dir\n"
        );
    }

    #[test]
    fn test04_archivo_con_objetos_agrega_nuevos_objetos() {
        clear_index();
        let logger = Rc::new(Logger::new(PathBuf::from("tmp/add_test04")).unwrap());
        let ubicacion = "test_file.txt".to_string();

        let mut add = Add::from(vec![ubicacion], logger.clone()).unwrap();
        add.ejecutar().unwrap();

        let ubicacion = "test_dir/objetos/archivo.txt".to_string();

        let mut add = Add::from(vec![ubicacion], logger.clone()).unwrap();
        add.ejecutar().unwrap();

        assert_eq!(add.index.len(), 2);

        let objeto = &add.index[0];

        if let Objeto::Blob(blob) = objeto {
            assert_eq!(blob.nombre, "test_file.txt");
            assert_eq!(blob.hash, "678e12dc5c03a7cf6e9f64e688868962ab5d8b65");
        }

        let objeto = &add.index[1];

        if let Objeto::Blob(blob) = objeto {
            assert_eq!(blob.nombre, "archivo.txt");
            assert_eq!(blob.hash, "2b824e648965b94c6c6b3dd0702feb91f699ed62");
        }

        let file = io::leer_a_string("./.gir/index").unwrap();

        assert_eq!(
            file,
            "40000 59fd3de622aa800bc7e0684773ce2dc40d876377 test_dir\n100644 678e12dc5c03a7cf6e9f64e688868962ab5d8b65 test_file.txt\n"
        );
    }

    #[test]
    fn test05_agregar_un_directorio_al_index() {
        clear_index();
        let logger = Rc::new(Logger::new(PathBuf::from("tmp/add_test05")).unwrap());

        let path = "test_dir/muchos_objetos".to_string();
        let mut add = Add::from(vec![path], logger.clone()).unwrap();
        add.ejecutar().unwrap();

        let file = io::leer_a_string("./.gir/index").unwrap();

        assert_eq!(
            file,
            "40000 9ce160c141eb52a2479bd734e54e0a64a49c5d76 test_dir\n"
        );
    }

    #[test]
    fn test06_editar_hijo_actualiza_padre() {
        clear_index();

        let logger = Rc::new(Logger::new(PathBuf::from("tmp/add_test06")).unwrap());

        let archivo_1 = "test_dir/muchos_objetos/archivo.txt".to_string();
        let mut add = Add::from(vec![archivo_1], logger.clone()).unwrap();
        add.ejecutar().unwrap();

        let file = io::leer_a_string("./.gir/index").unwrap();

        assert_eq!(
            file,
            "40000 dff341b050347f57726eb70b97addee11ea73194 test_dir\n"
        );

        let archivo_2 = "test_dir/muchos_objetos/archivo copy.txt".to_string();

        let mut add = Add::from(vec![archivo_2], logger.clone()).unwrap();
        add.ejecutar().unwrap();

        let file = io::leer_a_string("./.gir/index").unwrap();

        assert_eq!(
            file,
            "40000 9ce160c141eb52a2479bd734e54e0a64a49c5d76 test_dir\n"
        );
    }

    #[test]
    fn test07_agregar_dos_archivos() {
        clear_index();
        let logger = Rc::new(Logger::new(PathBuf::from("tmp/add_test07")).unwrap());
        let ubicacion = "test_file.txt".to_string();

        let ubicacion2 = "test_dir/objetos/archivo.txt".to_string();

        let mut add = Add::from(vec![ubicacion, ubicacion2], logger.clone()).unwrap();
        add.ejecutar().unwrap();

        assert_eq!(add.index.len(), 2);

        let objeto = &add.index[0];

        if let Objeto::Blob(blob) = objeto {
            assert_eq!(blob.nombre, "test_file.txt");
            assert_eq!(blob.hash, "678e12dc5c03a7cf6e9f64e688868962ab5d8b65");
        }

        let objeto = &add.index[1];

        if let Objeto::Blob(blob) = objeto {
            assert_eq!(blob.nombre, "archivo.txt");
            assert_eq!(blob.hash, "2b824e648965b94c6c6b3dd0702feb91f699ed62");
        }

        let file = io::leer_a_string("./.gir/index").unwrap();

        assert_eq!(
            file,
            "40000 59fd3de622aa800bc7e0684773ce2dc40d876377 test_dir\n100644 678e12dc5c03a7cf6e9f64e688868962ab5d8b65 test_file.txt\n"
        );
    }
}

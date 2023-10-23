use std::{path::PathBuf, rc::Rc};

use crate::{
    io::rm_directorio,
    tipos_de_dato::{
        logger::Logger,
        objeto::Objeto,
        utilidades_index::{crear_index, escribir_index, leer_index},
    },
};

pub struct Remove {
    pub ubicaciones: Vec<PathBuf>,
    pub cached: bool,
    pub logger: Rc<Logger>,
    pub index: Vec<Objeto>,
    pub recursivo: bool,
}

impl Remove {
    pub fn from(args: Vec<String>, logger: Rc<Logger>) -> Result<Remove, String> {
        crear_index();
        let index = leer_index()?;
        let mut ubicaciones: Vec<PathBuf> = Vec::new();
        let mut cached = false;
        let mut recursivo = false;

        for arg in args.iter() {
            match arg.as_str() {
                "--cached" => {
                    cached = true;
                }
                "-r" => {
                    recursivo = true;
                }
                ubicacion => {
                    ubicaciones.push(PathBuf::from(ubicacion));
                }
            }
        }

        Ok(Remove {
            logger,
            ubicaciones,
            index,
            cached,
            recursivo,
        })
    }

    pub fn ejecutar(&mut self) -> Result<String, String> {
        self.logger.log("Ejecutando remove".to_string());

        for ubicacion in self.ubicaciones.clone() {
            if ubicacion.is_dir() && !self.recursivo {
                Err("No se puede borrar un directorio sin la opcion -r".to_string())?;
            }

            let indice = self.index.iter().position(|x| match x {
                Objeto::Blob(blob) => blob.ubicacion == ubicacion,
                Objeto::Tree(tree) => {
                    tree.directorio == ubicacion || ubicacion.starts_with(&tree.directorio)
                }
            });

            if let Some(i) = indice {
                match &mut self.index[i] {
                    Objeto::Tree(ref mut tree) => {
                        if ubicacion.starts_with(&tree.directorio) {
                            tree.eliminar_hijo_por_directorio(&ubicacion);
                            escribir_index(self.logger.clone(), &self.index)?;
                        } else {
                            self.index.swap_remove(i);
                        }
                    }
                    Objeto::Blob(_) => {
                        self.index.swap_remove(i);
                    }
                }
            }

            if !self.cached {
                rm_directorio(ubicacion)?;
            }
        }

        escribir_index(self.logger.clone(), &self.index)?;

        Ok("".to_string())
    }
}

#[cfg(test)]
mod test {

    use crate::tipos_de_dato::comandos::add::Add;

    use super::*;

    fn crear_test_file() {
        let _ = std::fs::File::create("tmp/rm_test.txt");
    }

    fn existe_test_file() -> bool {
        PathBuf::from("tmp/rm_test.txt").exists()
    }

    fn clear_index() {
        let _ = std::fs::remove_file(".gir/index");
    }

    fn crear_test_dir() {
        let _ = std::fs::create_dir_all("tmp/test_dir/testfile.txt");
    }

    fn existe_test_dir() -> bool {
        PathBuf::from("tmp/test_dir").exists()
    }

    #[test]
    fn test01_remove_ejecutar() {
        clear_index();
        let logger = Rc::new(Logger::new(PathBuf::from("tmp/rm_test01")).unwrap());
        Add::from(vec!["test_file.txt".to_string()], logger.clone())
            .unwrap()
            .ejecutar()
            .unwrap();

        let args = vec!["--cached".to_string(), "test_file.txt".to_string()];
        let mut remove = Remove::from(args, logger).unwrap();
        remove.ejecutar().unwrap();
        let index = leer_index().unwrap();
        assert!(index.is_empty());
    }

    #[test]
    fn test02_remove_ejecutar_con_anidado() {
        clear_index();
        let logger = Rc::new(Logger::new(PathBuf::from("tmp/rm_test01")).unwrap());
        Add::from(
            vec!["test_dir/objetos/archivo.txt".to_string()],
            logger.clone(),
        )
        .unwrap()
        .ejecutar()
        .unwrap();

        let args = vec![
            "--cached".to_string(),
            "test_dir/objetos/archivo.txt".to_string(),
        ];
        let mut remove = Remove::from(args, logger).unwrap();
        remove.ejecutar().unwrap();
        let index = leer_index().unwrap();
        assert!(index.is_empty());
    }

    #[test]
    fn test03_remove_sin_cached() {
        clear_index();
        crear_test_file();
        let logger = Rc::new(Logger::new(PathBuf::from("tmp/rm_test01")).unwrap());
        Add::from(vec!["tmp/rm_test.txt".to_string()], logger.clone())
            .unwrap()
            .ejecutar()
            .unwrap();

        let args = vec!["tmp/rm_test.txt".to_string()];
        let mut remove = Remove::from(args, logger).unwrap();
        remove.ejecutar().unwrap();
        let index = leer_index().unwrap();
        assert!(index.is_empty());
        assert!(!existe_test_file());
    }

    #[test]
    fn test04_remove_recursivo() {
        clear_index();
        crear_test_dir();
        let logger = Rc::new(Logger::new(PathBuf::from("tmp/rm_test01")).unwrap());
        Add::from(vec!["tmp/test_dir".to_string()], logger.clone())
            .unwrap()
            .ejecutar()
            .unwrap();

        let args = vec!["-r".to_string(), "tmp/test_dir".to_string()];
        let mut remove = Remove::from(args, logger).unwrap();
        remove.ejecutar().unwrap();
        let index = leer_index().unwrap();
        assert!(index.is_empty());
        assert!(!existe_test_dir());
    }

    #[test]
    #[should_panic(expected = "No se puede borrar un directorio sin la opcion -r")]
    fn test05_remove_directorio_no_recursivo_falla() {
        clear_index();
        crear_test_dir();
        let logger = Rc::new(Logger::new(PathBuf::from("tmp/rm_test01")).unwrap());
        Add::from(vec!["tmp/test_dir".to_string()], logger.clone())
            .unwrap()
            .ejecutar()
            .unwrap();

        let args = vec!["tmp/test_dir".to_string()];
        let mut remove = Remove::from(args, logger).unwrap();
        remove.ejecutar().unwrap();
    }
}

use std::{collections::HashSet, path::PathBuf, rc::Rc};

use crate::{
    io::rm_directorio,
    tipos_de_dato::{logger::Logger, objeto::Objeto},
    utilidades_index::{crear_index, escribir_index, leer_index, ObjetoIndex},
};

pub struct Remove {
    pub ubicaciones: Vec<PathBuf>,
    pub cached: bool,
    pub logger: Rc<Logger>,
    pub index: Vec<ObjetoIndex>,
}

impl Remove {
    fn limpiar_directorios_vacios(&self) {
        let mut ubicaciones_a_corroborar: HashSet<PathBuf> = HashSet::new();

        for ubicacion in self.ubicaciones.clone() {
            let mut ubicacion_actual = ubicacion.clone();
            loop {
                ubicacion_actual = ubicacion_actual.parent().unwrap().to_path_buf();
                if ubicacion_actual == PathBuf::from("") {
                    break;
                }
                if ubicaciones_a_corroborar.contains(&ubicacion_actual) {
                    break;
                }
                ubicaciones_a_corroborar.insert(ubicacion_actual.clone());
            }
        }

        for ubicacion in ubicaciones_a_corroborar {
            if !ubicacion.is_dir() {
                continue;
            }

            let hijos = std::fs::read_dir(&ubicacion)
                .map_err(|_| "Error al obtener hijos de directorio".to_string())
                .unwrap();

            if hijos.count() == 0 {
                rm_directorio(ubicacion).unwrap();
            }
        }
    }

    pub fn obtener_ubicaciones_hoja(
        ubicaciones: Vec<PathBuf>,
        recursivo: bool,
    ) -> Result<Vec<PathBuf>, String> {
        let mut ubicaciones_hoja: Vec<PathBuf> = Vec::new();
        for ubicacion in ubicaciones {
            if ubicacion.is_file() {
                ubicaciones_hoja.push(ubicacion);
            } else if ubicacion.is_dir() {
                if !recursivo {
                    Err("No se puede borrar un directorio sin la opcion -r".to_string())?;
                }
                let mut directorios = std::fs::read_dir(ubicacion)
                    .map_err(|_| "Error al obtener directorios hoja".to_string())?;
                while let Some(Ok(directorio)) = directorios.next() {
                    let path = directorio.path();
                    if path.is_file() {
                        ubicaciones_hoja.push(path);
                    } else if path.is_dir() {
                        ubicaciones_hoja
                            .append(&mut Self::obtener_ubicaciones_hoja(vec![path], true)?);
                    }
                }
            }
        }
        Ok(ubicaciones_hoja)
    }

    pub fn from(args: Vec<String>, logger: Rc<Logger>) -> Result<Remove, String> {
        crear_index();
        let index = leer_index()?;
        let mut ubicaciones_recibidas: Vec<PathBuf> = Vec::new();
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
                    ubicaciones_recibidas.push(PathBuf::from(ubicacion));
                }
            }
        }

        let ubicaciones: Vec<PathBuf> =
            Self::obtener_ubicaciones_hoja(ubicaciones_recibidas, recursivo)?;

        Ok(Remove {
            logger,
            ubicaciones,
            index,
            cached,
        })
    }

    pub fn ejecutar(&mut self) -> Result<String, String> {
        self.logger.log("Ejecutando remove".to_string());

        for ubicacion in self.ubicaciones.clone() {
            if ubicacion.is_dir() {
                Err("No se puede borrar un directorio sin la opcion -r".to_string())?;
            }

            let nuevo_objeto = Objeto::from_directorio(ubicacion.clone(), None)?;
            let nuevo_objeto_index = ObjetoIndex {
                merge: false,
                es_eliminado: true,
                objeto: nuevo_objeto.clone(),
            };

            let indice = self
                .index
                .iter()
                .position(|objeto_index| match objeto_index.objeto {
                    Objeto::Blob(ref blob) => blob.ubicacion == ubicacion,
                    Objeto::Tree(_) => false,
                });

            if indice.is_some() {
                Err("No se puede borrar un archivo en el index".to_string())?;
            } else {
                self.index.push(nuevo_objeto_index);
            }

            if !self.cached {
                rm_directorio(ubicacion)?;
            }
        }

        escribir_index(self.logger.clone(), &self.index)?;
        self.limpiar_directorios_vacios();

        Ok("".to_string())
    }
}

#[cfg(test)]
mod test {

    use crate::{
        io::{escribir_bytes, leer_a_string},
        tipos_de_dato::comandos::{add::Add, commit::Commit},
    };

    use super::*;

    fn crear_test_file() {
        escribir_bytes("tmp/rm_test.txt", "contenido").unwrap();
    }

    fn existe_test_file() -> bool {
        PathBuf::from("tmp/rm_test.txt").exists()
    }

    fn clear_index() {
        let _ = std::fs::remove_file(".gir/index");
    }

    fn crear_archivo_en_dir() {
        escribir_bytes("tmp/test_dir/testfile.txt", "contenido").unwrap();
    }

    fn existe_archivo_en_dir() -> bool {
        PathBuf::from("tmp/test_dir/testfile.txt").exists()
    }

    fn existe_dir() -> bool {
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

        Commit::from(
            &mut vec!["-m".to_string(), "mensaje".to_string()],
            logger.clone(),
        )
        .unwrap()
        .ejecutar()
        .unwrap();

        let args = vec!["--cached".to_string(), "test_file.txt".to_string()];
        Remove::from(args, logger).unwrap().ejecutar().unwrap();

        let index = leer_a_string(".gir/index").unwrap();
        assert_eq!(
            index,
            "- 0 100644 678e12dc5c03a7cf6e9f64e688868962ab5d8b65 test_file.txt\n"
        )
    }

    #[test]
    fn test02_remove_recursivo() {
        clear_index();
        crear_archivo_en_dir();
        let logger = Rc::new(Logger::new(PathBuf::from("tmp/rm_test01")).unwrap());
        Add::from(vec!["tmp/test_dir".to_string()], logger.clone())
            .unwrap()
            .ejecutar()
            .unwrap();

        Commit::from(
            &mut vec!["-m".to_string(), "mensaje".to_string()],
            logger.clone(),
        )
        .unwrap()
        .ejecutar()
        .unwrap();

        let args = vec![
            "--cached".to_string(),
            "-r".to_string(),
            "tmp/test_dir".to_string(),
        ];
        Remove::from(args, logger).unwrap().ejecutar().unwrap();

        let index = leer_a_string(".gir/index").unwrap();

        assert_eq!(
            index,
            "- 0 100644 d2207d7532b976e05bada36e723b79f26cd7f2cd tmp/test_dir/testfile.txt\n"
        )
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

        Commit::from(
            &mut vec!["-m".to_string(), "mensaje".to_string()],
            logger.clone(),
        )
        .unwrap()
        .ejecutar()
        .unwrap();

        let args = vec!["tmp/rm_test.txt".to_string()];
        let mut remove = Remove::from(args, logger).unwrap();
        remove.ejecutar().unwrap();

        let index = leer_a_string(".gir/index").unwrap();

        assert_eq!(
            index,
            "- 0 100644 d2207d7532b976e05bada36e723b79f26cd7f2cd tmp/rm_test.txt\n"
        );
        assert!(!existe_test_file());
    }

    #[test]
    fn test04_remove_recursivo_sin_cached() {
        clear_index();
        crear_archivo_en_dir();

        let logger = Rc::new(Logger::new(PathBuf::from("tmp/rm_test01")).unwrap());
        Add::from(vec!["tmp/test_dir".to_string()], logger.clone())
            .unwrap()
            .ejecutar()
            .unwrap();

        Commit::from(
            &mut vec!["-m".to_string(), "mensaje".to_string()],
            logger.clone(),
        )
        .unwrap()
        .ejecutar()
        .unwrap();

        let args = vec!["-r".to_string(), "tmp/test_dir".to_string()];
        Remove::from(args, logger).unwrap().ejecutar().unwrap();

        let index = leer_a_string(".gir/index").unwrap();

        assert_eq!(
            index,
            "- 0 100644 d2207d7532b976e05bada36e723b79f26cd7f2cd tmp/test_dir/testfile.txt\n"
        );

        assert!(!existe_archivo_en_dir());
        assert!(!existe_dir());
    }

    #[test]
    #[should_panic(expected = "No se puede borrar un directorio sin la opcion -r")]
    fn test05_remove_directorio_no_recursivo_falla() {
        clear_index();
        crear_archivo_en_dir();

        let logger = Rc::new(Logger::new(PathBuf::from("tmp/rm_test01")).unwrap());
        Add::from(vec!["tmp/test_dir".to_string()], logger.clone())
            .unwrap()
            .ejecutar()
            .unwrap();

        Commit::from(
            &mut vec!["-m".to_string(), "mensaje".to_string()],
            logger.clone(),
        )
        .unwrap()
        .ejecutar()
        .unwrap();

        let args = vec!["tmp/test_dir".to_string()];
        Remove::from(args, logger).unwrap().ejecutar().unwrap();
    }
}

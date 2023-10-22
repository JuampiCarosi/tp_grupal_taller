use std::{fs, path::PathBuf, rc::Rc};

use super::{
    comandos::hash_object::HashObject,
    logger,
    objetos::{blob::Blob, tree::Tree},
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Objeto {
    Tree(Tree),
    Blob(Blob),
}

pub fn flag_es_un_objeto_(flag: &String) -> bool {
    flag == "blob" || flag == "tree" || flag == "commit" || flag == "tag"
}

impl Objeto {
    pub fn obtener_hash(&self) -> String {
        match self {
            Objeto::Tree(tree) => match tree.obtener_hash() {
                Ok(hash) => hash,
                Err(err) => err.to_string(),
            },
            Objeto::Blob(blob) => blob.obtener_hash(),
        }
    }

    pub fn obtener_tamanio(&self) -> Result<usize, String> {
        match self {
            Objeto::Tree(tree) => Ok(tree.obtener_tamanio()?),
            Objeto::Blob(blob) => Ok(blob.obtener_tamanio()?),
        }
    }

    pub fn from_index(linea_index: String) -> Result<Objeto, String> {
        let mut line = linea_index.split_whitespace();

        let modo = line.next().unwrap_or_else(|| "Error al leer el modo");
        let hash = line
            .next()
            .unwrap_or_else(|| "Error al leer el nombre del hash");
        let ubicacion_string = line
            .next()
            .unwrap_or_else(|| "Error al leer el nombre del archivo");
        let ubicacion = PathBuf::from(ubicacion_string);
        let nombre = ubicacion_string
            .split('/')
            .last()
            .unwrap_or_else(|| "Error al leer el nombre del archivo");

        match modo {
            "100644" => Ok(Objeto::Blob(Blob {
                nombre: nombre.to_string(),
                ubicacion,
                hash: hash.to_string(),
            })),
            "40000" => {
                let tree = Tree::from_hash_20(hash.to_string(), ubicacion_string.to_string());
                Ok(Objeto::Tree(tree))
            }
            _ => Err("Modo no soportado".to_string()),
        }
    }

    pub fn from_directorio(directorio: String) -> Result<Objeto, String> {
        let mut objetos: Vec<Objeto> = Vec::new();

        let metadata = match fs::metadata(&directorio) {
            Ok(metadata) => metadata,
            Err(_) => Err(format!("No se pudo leer el directorio {directorio}"))?,
        };

        if metadata.is_dir() {
            for entrada in fs::read_dir(&directorio).unwrap() {
                let entrada = entrada.unwrap();
                let path = entrada.path();
                let path = path.to_str().unwrap().to_string();

                if path.contains(".DS_Store") {
                    continue;
                }

                let objeto = match fs::metadata(&path) {
                    Ok(_) => Objeto::from_directorio(path)?,
                    Err(_) => Err("Error al leer el archivo".to_string())?,
                };
                objetos.push(objeto);
            }

            Ok(Objeto::Tree(Tree {
                directorio,
                objetos,
            }))
        } else if fs::metadata(&directorio).unwrap().is_file() {
            let logger = Rc::new(logger::Logger::new()?);
            let hash = HashObject {
                logger,
                escribir: false,
                nombre_archivo: directorio.clone(),
            }
            .ejecutar()
            .unwrap();

            let directorio_split = directorio.split('/').collect::<Vec<&str>>();
            let nombre = directorio_split.last().unwrap().to_string();
            Ok(Objeto::Blob(Blob {
                nombre,
                hash,
                ubicacion: PathBuf::from(directorio),
            }))
        } else {
            Err("No se pudo leer el directorio".to_string())
        }
    }

    fn esta_directorio_habilitado(
        directorio: &String,
        directorios_habilitados: &Vec<String>,
    ) -> bool {
        for directorio_habilitado in directorios_habilitados {
            if directorio.starts_with(directorio_habilitado)
                || directorio_habilitado.starts_with(directorio)
            {
                return true;
            }
        }
        return false;
    }

    pub fn from_directorio_con_hijos_especificados(
        directorio: String,
        directorios_habilitados: &Vec<String>,
    ) -> Result<Objeto, String> {
        let mut objetos: Vec<Objeto> = Vec::new();

        let metadata = match fs::metadata(&directorio) {
            Ok(metadata) => metadata,
            Err(_) => Err(format!("No se pudo leer el directorio {directorio}"))?,
        };

        if metadata.is_dir() {
            for entrada in fs::read_dir(&directorio).unwrap() {
                let entrada = entrada.unwrap();
                let path = entrada.path();
                let path = path.to_str().unwrap().to_string();

                if path.contains(".DS_Store")
                    || !Self::esta_directorio_habilitado(&path, directorios_habilitados)
                {
                    continue;
                }

                let objeto = match fs::metadata(&path) {
                    Ok(_) => {
                        let objt = Objeto::from_directorio_con_hijos_especificados(
                            path,
                            directorios_habilitados,
                        )?;
                        objt
                    }
                    Err(_) => Err("Error al leer el archivo".to_string())?,
                };

                objetos.push(objeto);
            }

            Ok(Objeto::Tree(Tree {
                directorio,
                objetos,
            }))
        } else if fs::metadata(&directorio).unwrap().is_file() {
            let logger = Rc::new(logger::Logger::new()?);
            let hash = HashObject {
                logger,
                escribir: false,
                nombre_archivo: directorio.clone(),
            }
            .ejecutar()
            .unwrap();

            let directorio_split = directorio.split('/').collect::<Vec<&str>>();
            let nombre = directorio_split.last().unwrap().to_string();
            Ok(Objeto::Blob(Blob {
                nombre,
                hash,
                ubicacion: PathBuf::from(directorio),
            }))
        } else {
            Err("No se pudo leer el directorio".to_string())
        }
    }
}

#[cfg(test)]

mod test {

    use super::*;

    #[test]
    fn test01_blob_from_index() {
        let objeto = Objeto::from_index("100644 1234567890 ./hola.txt".to_string()).unwrap();
        assert_eq!(
            objeto,
            Objeto::Blob(Blob {
                nombre: "hola.txt".to_string(),
                hash: "1234567890".to_string(),
                ubicacion: PathBuf::from("./hola.txt"),
            })
        );
    }

    #[test]
    fn test02_blob_from_directorio() {
        let objeto = Objeto::from_directorio("test_dir/objetos/archivo.txt".to_string()).unwrap();

        assert_eq!(
            objeto,
            Objeto::Blob(Blob {
                nombre: "archivo.txt".to_string(),
                hash: "2b824e648965b94c6c6b3dd0702feb91f699ed62".to_string(),
                ubicacion: PathBuf::from("test_dir/objetos/archivo.txt"),
            })
        );
    }

    #[test]

    fn test03_tree_from_directorio() {
        let objeto = Objeto::from_directorio("test_dir/objetos".to_string()).unwrap();

        let hijo = Objeto::Blob(Blob {
            nombre: "archivo.txt".to_string(),
            hash: "2b824e648965b94c6c6b3dd0702feb91f699ed62".to_string(),
            ubicacion: PathBuf::from("test_dir/objetos/archivo.txt"),
        });

        assert_eq!(
            objeto,
            Objeto::Tree(Tree {
                directorio: "test_dir/objetos".to_string(),
                objetos: vec![hijo]
            })
        );
    }

    #[test]
    fn test04_tree_from_index() {
        let objeto_a_escibir = Objeto::from_directorio("test_dir/objetos".to_string()).unwrap();

        if let Objeto::Tree(ref tree) = objeto_a_escibir {
            tree.escribir_en_base().unwrap();
        } else {
            panic!("No se pudo leer el directorio");
        }

        let objeto = Objeto::from_index(format!(
            "40000 {} test_dir/objetos",
            &objeto_a_escibir.obtener_hash()[..20]
        ))
        .unwrap();

        let hijo = Objeto::Blob(Blob {
            nombre: "archivo.txt".to_string(),
            hash: "2b824e648965b94c6c6b3dd0702feb91f699ed62".to_string(),
            ubicacion: PathBuf::from("test_dir/objetos/archivo.txt"),
        });

        assert_eq!(
            objeto,
            Objeto::Tree(Tree {
                directorio: "test_dir/objetos".to_string(),
                objetos: vec![hijo]
            })
        );
    }
}

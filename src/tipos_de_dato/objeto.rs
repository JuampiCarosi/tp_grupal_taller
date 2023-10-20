use std::{fs, path::PathBuf, rc::Rc};

use super::{
    comando::Comando,
    comandos::hash_object::HashObject,
    logger,
    objetos::{blob::Blob, tree::Tree},
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Objeto {
    Tree(Tree),
    Blob(Blob),
}

impl Objeto {
    pub fn obtener_hash(&self) -> String {
        match self {
            Objeto::Tree(tree) => tree.obtener_hash(),
            Objeto::Blob(blob) => blob.obtener_hash(),
        }
    }

    pub fn obtener_tamanio(&self) -> usize {
        match self {
            Objeto::Tree(tree) => tree.obtener_tamanio(),
            Objeto::Blob(blob) => blob.obtener_tamanio(),
        }
    }

    pub fn from_index(linea_index: String) -> Result<Objeto, String> {
        let mut line = linea_index.split_whitespace();

        let modo = line.next().unwrap();
        let hash = line.next().unwrap();
        let nombre = line.next().unwrap();

        match modo {
            "100644" => Ok(Objeto::Blob(Blob {
                nombre: nombre.to_string(),
                hash: hash.to_string(),
            })),
            "40000" => Ok(Self::from_directorio(nombre.to_string())?),
            _ => Err("Modo no soportado".to_string()),
        }
    }

    pub fn from_directorio(directorio: String) -> Result<Objeto, String> {
        let mut objetos: Vec<Objeto> = Vec::new();

        if PathBuf::from(&directorio).is_dir() {
            for entrada in fs::read_dir(&directorio).unwrap() {
                let entrada = entrada.unwrap();
                let path = entrada.path();
                let path = path.to_str().unwrap().to_string();

                let objeto = match fs::metadata(&path) {
                    Ok(_) => Objeto::from_directorio(path)?,
                    Err(_) => Err("Error al leer el archivo".to_string())?,
                };
                objetos.push(objeto);
            }

            return Ok(Objeto::Tree(Tree {
                directorio,
                objetos,
            }));
        } else {
            let logger = Rc::new(logger::Logger::new()?);
            let hash =
                Comando::HashObject(HashObject::from(&mut vec![directorio.clone()], logger)?)
                    .ejecutar()
                    .unwrap();

            let directorio_split = directorio.split('/').collect::<Vec<&str>>();
            let nombre = directorio_split.last().unwrap().to_string();
            return Ok(Objeto::Blob(Blob { nombre, hash }));
        }
    }
}

#[cfg(test)]

mod test {

    use super::*;

    #[test]
    fn blob_from_index() {
        let objeto = Objeto::from_index("100644 1234567890 hola.txt".to_string()).unwrap();
        assert_eq!(
            objeto,
            Objeto::Blob(Blob {
                nombre: "hola.txt".to_string(),
                hash: "1234567890".to_string()
            })
        );
    }

    #[test]
    fn blob_from_directorio() {
        let objeto = Objeto::from_directorio("test_dir/objetos/archivo.txt".to_string()).unwrap();

        assert_eq!(
            objeto,
            Objeto::Blob(Blob {
                nombre: "archivo.txt".to_string(),
                hash: "2b824e648965b94c6c6b3dd0702feb91f699ed62".to_string()
            })
        );
    }

    #[test]

    fn tree_from_directorio() {
        let objeto = Objeto::from_directorio("test_dir/objetos".to_string()).unwrap();

        let hijo = Objeto::Blob(Blob {
            nombre: "archivo.txt".to_string(),
            hash: "2b824e648965b94c6c6b3dd0702feb91f699ed62".to_string(),
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
    fn tree_from_index() {
        let objeto = Objeto::from_index("40000 1234567890 test_dir/objetos".to_string()).unwrap();

        let hijo = Objeto::Blob(Blob {
            nombre: "archivo.txt".to_string(),
            hash: "2b824e648965b94c6c6b3dd0702feb91f699ed62".to_string(),
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

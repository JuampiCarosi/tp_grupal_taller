use std::path::PathBuf;

use super::objetos::{blob::Blob, tree::Tree};

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

        let modo = line.next().ok_or("Error al leer el modo")?;
        let hash = line.next().ok_or("Error al leer el hash")?;
        let ubicacion_string = line.next().ok_or("Error al leer la ubicacion")?;

        let ubicacion = PathBuf::from(ubicacion_string);
        let nombre = match ubicacion_string.split('/').last() {
            Some(nombre) => nombre,
            None => Err("Error al leer el nombre")?,
        };

        match modo {
            "100644" => Ok(Objeto::Blob(Blob {
                nombre: nombre.to_string(),
                ubicacion,
                hash: hash.to_string(),
            })),
            "40000" => {
                let tree = Tree::from_hash(hash.to_string(), ubicacion)?;
                Ok(Objeto::Tree(tree))
            }
            _ => Err("Modo no soportado".to_string()),
        }
    }

    pub fn from_directorio(
        directorio: PathBuf,
        hijos_especificados: Option<&Vec<PathBuf>>,
    ) -> Result<Objeto, String> {
        if directorio.is_dir() {
            let tree = Tree::from_directorio(directorio.clone(), hijos_especificados)?;
            Ok(Objeto::Tree(tree))
        } else if directorio.is_file() {
            let blob = Blob::from_directorio(directorio.clone())?;
            Ok(Objeto::Blob(blob))
        } else {
            Err(format!("No se pudo leer el directorio {directorio:#?}"))
        }
    }

    pub fn esta_directorio_habilitado(
        directorio: &PathBuf,
        directorios_habilitados: &Vec<PathBuf>,
    ) -> bool {
        for directorio_habilitado in directorios_habilitados {
            if directorio.starts_with(directorio_habilitado)
                || directorio_habilitado.starts_with(directorio)
            {
                return true;
            }
        }
        false
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
        let objeto =
            Objeto::from_directorio(PathBuf::from("test_dir/objetos/archivo.txt"), None).unwrap();

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
        let objeto = Objeto::from_directorio(PathBuf::from("test_dir/objetos"), None).unwrap();

        let hijo = Objeto::Blob(Blob {
            nombre: "archivo.txt".to_string(),
            hash: "2b824e648965b94c6c6b3dd0702feb91f699ed62".to_string(),
            ubicacion: PathBuf::from("test_dir/objetos/archivo.txt"),
        });

        assert_eq!(
            objeto,
            Objeto::Tree(Tree {
                directorio: PathBuf::from("test_dir/objetos"),
                objetos: vec![hijo]
            })
        );
    }

    #[test]
    fn test04_tree_from_index() {
        let objeto_a_escibir =
            Objeto::from_directorio(PathBuf::from("test_dir/objetos"), None).unwrap();

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
                directorio: PathBuf::from("test_dir/objetos"),
                objetos: vec![hijo]
            })
        );
    }
}

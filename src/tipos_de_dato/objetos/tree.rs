use std::fmt::Display;

use sha1::{Digest, Sha1};

use crate::tipos_de_dato::objeto::Objeto;

#[derive(Clone, Debug, PartialEq, Eq)]

pub struct Tree {
    pub directorio: String,
    pub objetos: Vec<Objeto>,
}
impl Tree {
    pub fn obtener_tamanio(&self) -> Result<usize, String> {
        let contenido = match Self::mostrar_contenido(&self.objetos) {
            Ok(contenido) => contenido,
            Err(_) => return Err("No se pudo obtener el contenido del tree".to_string()),
        };
        Ok(contenido.len())
    }

    pub fn contiene_hijo(&self, hash_hijo: String) -> bool {
        for objeto in &self.objetos {
            if objeto.obtener_hash() == hash_hijo {
                return true;
            }
        }
        false
    }

    pub fn actualizar_hijos(&mut self, hash_hijo: String) {
        for objeto in &mut self.objetos {
            if objeto.obtener_hash() == hash_hijo {
                match objeto {
                    Objeto::Tree(tree) => {
                        tree.actualizar_hijos(hash_hijo.clone());
                    }
                    Objeto::Blob(blob) => {
                        blob.hash = hash_hijo.clone();
                    }
                }
            }
        }
    }

    pub fn obtener_hash(&self) -> Result<String, String> {
        let contenido = Self::mostrar_contenido(&self.objetos)?;
        let header = format!("tree {}\0", contenido.len());

        let contenido_total = format!("{}{}", header, contenido);

        let mut hasher = Sha1::new();
        hasher.update(contenido_total);
        let hash = hasher.finalize();
        Ok(format!("{:x}", hash))
    }

    fn ordenar_objetos_alfabeticamente(objetos: &[Objeto]) -> Vec<Objeto> {
        let mut objetos = objetos.to_owned();
        objetos.sort_by(|a, b| match (a, b) {
            (Objeto::Blob(a), Objeto::Blob(b)) => a.nombre.cmp(&b.nombre),
            (Objeto::Tree(a), Objeto::Tree(b)) => a.directorio.cmp(&b.directorio),
            (Objeto::Blob(a), Objeto::Tree(b)) => a.nombre.cmp(&b.directorio),
            (Objeto::Tree(a), Objeto::Blob(b)) => a.directorio.cmp(&b.nombre),
        });
        objetos
    }

    fn mostrar_contenido(objetos: &Vec<Objeto>) -> Result<String, String> {
        let mut output = String::new();

        let objetos_ordenados = Self::ordenar_objetos_alfabeticamente(objetos);

        for objeto in objetos_ordenados {
            let line = match objeto {
                Objeto::Blob(blob) => format!("100644 {}\0{}", blob.nombre, &blob.hash[..20]),
                Objeto::Tree(tree) => {
                    let name = match tree.directorio.split('/').last() {
                        Some(name) => name,
                        None => return Err("Error al obtener el nombre del directorio".to_string()),
                    };
                    let hash = tree.obtener_hash()?;
                    format!("40000 {}\0{}", name, &hash[..20])
                }
            };
            output.push_str(&line);
        }
        Ok(output)
    }
}

impl Display for Tree {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self.directorio.split("/").last() {
            Some(name) => name,
            None => return Err(std::fmt::Error),
        };
        let hash = match self.obtener_hash() {
            Ok(hash) => hash,
            Err(_) => return Err(std::fmt::Error),
        };
        let string = format!("40000 {} {}\n", hash, name);
        write!(f, "{}", string)
    }
}

#[cfg(test)]

mod test {
    use crate::tipos_de_dato::{objeto::Objeto, objetos::tree::Tree};

    #[test]
    fn test01_test_obtener_hash() {
        let objeto = Objeto::from_directorio("test_dir/objetos".to_string()).unwrap();
        let hash = objeto.obtener_hash();
        assert_eq!(hash, "bf902127ac66b999327fba07a9f4b7a50b87922a");
    }

    #[test]
    fn test02_test_obtener_tamanio() {
        let objeto = Objeto::from_directorio("test_dir/muchos_objetos".to_string()).unwrap();
        let tamanio = objeto.obtener_tamanio().unwrap();
        assert_eq!(tamanio, 83);
    }

    #[test]
    fn test03_test_mostrar_contenido() {
        let objeto = Objeto::from_directorio("test_dir/objetos".to_string()).unwrap();

        if let Objeto::Tree(tree) = objeto {
            let contenido = Tree::mostrar_contenido(&tree.objetos).unwrap();
            assert_eq!(contenido, "100644 archivo.txt\02b824e648965b94c6c6b");
        } else {
            assert!(false)
        }
    }

    #[test]
    fn test04_test_mostrar_contenido_recursivo() {
        let objeto = Objeto::from_directorio("test_dir/".to_string()).unwrap();

        if let Objeto::Tree(tree) = objeto {
            let contenido = Tree::mostrar_contenido(&tree.objetos).unwrap();
            assert_eq!(
                contenido,
                "40000 muchos_objetos\0748ef9d5f9df6f40b07b40000 objetos\0bf902127ac66b999327f"
            );
        } else {
            assert!(false)
        }
    }
}

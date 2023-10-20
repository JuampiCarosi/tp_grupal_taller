use std::fmt::Display;

use sha1::{Digest, Sha1};

use crate::tipos_de_dato::objeto::Objeto;

#[derive(Clone, Debug, PartialEq, Eq)]

pub struct Tree {
    pub directorio: String,
    pub objetos: Vec<Objeto>,
}
impl Tree {
    pub fn obtener_tamanio(&self) -> usize {
        return Self::mostrar_contenido(&self.objetos).len();
    }

    pub fn contiene_hijo(&self, hash_hijo: String) -> bool {
        for objeto in &self.objetos {
            if objeto.obtener_hash() == hash_hijo {
                return true;
            }
        }
        return false;
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

    pub fn obtener_hash(&self) -> String {
        let contenido = Self::mostrar_contenido(&self.objetos);
        let header = format!("tree {}\0", contenido.len());

        let contenido_total = format!("{}{}", header, contenido);
        println!("{}", contenido_total);

        let mut hasher = Sha1::new();
        hasher.update(contenido_total);
        let hash = hasher.finalize();
        format!("{:x}", hash)
    }

    fn ordenar_objetos_alfabeticamente(objetos: &Vec<Objeto>) -> Vec<Objeto> {
        let mut objetos = objetos.clone();
        objetos.sort_by(|a, b| match (a, b) {
            (Objeto::Blob(a), Objeto::Blob(b)) => a.nombre.cmp(&b.nombre),
            (Objeto::Tree(a), Objeto::Tree(b)) => a.directorio.cmp(&b.directorio),
            (Objeto::Blob(a), Objeto::Tree(b)) => a.nombre.cmp(&b.directorio),
            (Objeto::Tree(a), Objeto::Blob(b)) => a.directorio.cmp(&b.nombre),
        });
        objetos
    }

    fn mostrar_contenido(objetos: &Vec<Objeto>) -> String {
        let mut output = String::new();

        let objetos_ordenados = Self::ordenar_objetos_alfabeticamente(objetos);

        for objeto in objetos_ordenados {
            let line = match objeto {
                Objeto::Blob(blob) => format!("100644 {}\0{}", blob.nombre, &blob.hash[..20]),
                Objeto::Tree(tree) => {
                    let name = tree.directorio.split("/").last().unwrap();
                    format!("40000 {}\0{}", name, &tree.obtener_hash()[..20])
                }
            };
            output.push_str(&line);
        }
        return output;
    }
}

impl Display for Tree {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = self.directorio.split("/").last().unwrap();
        let string = format!("40000 {} {}\n", self.obtener_hash(), name);
        write!(f, "{}", string)
    }
}

#[cfg(test)]

mod test {
    use crate::tipos_de_dato::{objeto::Objeto, objetos::tree::Tree};

    #[test]
    fn test_obtener_hash() {
        let objeto = Objeto::from_directorio("test_dir/objetos".to_string()).unwrap();
        let hash = objeto.obtener_hash();
        assert_eq!(hash, "bf902127ac66b999327fba07a9f4b7a50b87922a");
    }

    #[test]
    fn test_obtener_tamanio() {
        let objeto = Objeto::from_directorio("test_dir/muchos_objetos".to_string()).unwrap();
        let tamanio = objeto.obtener_tamanio();
        assert_eq!(tamanio, 83);
    }

    #[test]
    fn test_mostrar_contenido() {
        let objeto = Objeto::from_directorio("test_dir/objetos".to_string()).unwrap();

        if let Objeto::Tree(tree) = objeto {
            let contenido = Tree::mostrar_contenido(&tree.objetos);
            assert_eq!(contenido, "100644 archivo.txt\02b824e648965b94c6c6b");
        } else {
            assert!(false)
        }
    }

    #[test]
    fn test_mostrar_contenido_recursivo() {
        let objeto = Objeto::from_directorio("test_dir/".to_string()).unwrap();

        if let Objeto::Tree(tree) = objeto {
            let contenido = Tree::mostrar_contenido(&tree.objetos);
            assert_eq!(
                contenido,
                "40000 muchos_objetos\0748ef9d5f9df6f40b07b40000 objetos\0bf902127ac66b999327f"
            );
        } else {
            assert!(false)
        }
    }
}

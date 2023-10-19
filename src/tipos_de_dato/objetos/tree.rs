use sha1::{Digest, Sha1};

use crate::tipos_de_dato::objeto::Objeto;

#[derive(Clone, Debug, PartialEq, Eq)]

pub struct Tree {
    pub directorio: String,
    pub objetos: Vec<Objeto>,
}

impl Tree {
    pub fn obtener_tamanio(&self) -> usize {
        let mut tamanio = 0;
        for objeto in &self.objetos {
            tamanio += objeto.obtener_tamanio();
        }
        tamanio
    }

    pub fn obtener_hash(&self) -> String {
        let header = format!("tree {}\0", self.directorio.len());
        let contenido = Self::mostrar_contenido(self.objetos.clone());

        let contenido_total = header + &contenido;
        let mut hasher = Sha1::new();

        hasher.update(contenido_total);
        let hash = hasher.finalize();
        return format!("{:x}", hash);
    }

    fn mostrar_contenido(objetos: Vec<Objeto>) -> String {
        let mut output = String::new();

        for objeto in objetos {
            let line = match objeto {
                Objeto::Blob(blob) => format!("100644 blob {} {}\0", blob.hash, blob.nombre),
                Objeto::Tree(tree) => {
                    format!("40000 tree {} {}\0", tree.obtener_hash(), tree.directorio)
                }
            };
            output.push_str(&line);
        }

        return output;
    }
}

#[cfg(test)]

mod test {
    use crate::tipos_de_dato::{objeto::Objeto, objetos::tree::Tree};

    // #[test]
    // fn test_obtener_hash() {
    //     let objeto = Objeto::from_directorio("test_dir/objetos".to_string()).unwrap();
    //     let hash = objeto.obtener_hash();
    //     assert_eq!(hash, "dd167861ef3613e833d23d9519c9e2a79bee25f2");
    // }

    // #[test]
    // fn test_obtener_tamanio() {
    //     let objeto = Objeto::from_directorio("test_dir/objetos".to_string()).unwrap();
    //     let tamanio = objeto.obtener_tamanio();
    //     assert_eq!(tamanio, 39);
    // }

    #[test]
    fn test_mostrar_contenido() {
        let objeto = Objeto::from_directorio("test_dir/objetos".to_string()).unwrap();

        if let Objeto::Tree(tree) = objeto {
            let contenido = Tree::mostrar_contenido(tree.objetos);
            assert_eq!(
                contenido,
                "100644 blob 2b824e648965b94c6c6b3dd0702feb91f699ed62 archivo.txt\0"
            );
        } else {
            assert!(false)
        }
    }
}

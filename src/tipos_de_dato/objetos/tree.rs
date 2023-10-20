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

    fn mostrar_contenido(objetos: &Vec<Objeto>) -> String {
        let mut output = String::new();

        for objeto in objetos {
            let line = match objeto {
                Objeto::Blob(blob) => format!("100644 {}\0{}", blob.nombre, &blob.hash[..20]),
                Objeto::Tree(tree) => {
                    format!("40000 {}\0{}", tree.directorio, &tree.obtener_hash()[..20])
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

    #[test]
    fn test_obtener_hash() {
        let objeto = Objeto::from_directorio("test_dir/objetos".to_string()).unwrap();
        let hash = objeto.obtener_hash();
        assert_eq!(hash, "1442e275fd3a2e743f6bccf3b11ab27862157179");
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
            assert_eq!(
                contenido,
                "100644 blob 2b824e648965b94c6c6b3dd0702feb91f699ed62 archivo.txt\0"
            );
        } else {
            assert!(false)
        }
    }
}

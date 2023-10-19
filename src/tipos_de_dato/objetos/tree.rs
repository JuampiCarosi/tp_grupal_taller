use sha1::{Digest, Sha1};

use crate::tipos_de_dato::objeto::Objeto;

#[derive(Clone)]

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
                Objeto::Blob(blob) => format!("100644 blob  {} {}\0", blob.hash, blob.nombre),
                Objeto::Tree(tree) => {
                    format!("40000 tree {} {}\0", tree.obtener_hash(), tree.directorio)
                }
            };
            output.push_str(&line);
        }
        return output;
    }
}

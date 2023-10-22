use std::{fmt::Display, fs, io::Write, path::PathBuf, rc::Rc};

use flate2::{self, write::ZlibEncoder, Compression};

use sha1::{Digest, Sha1};

use crate::{
    io,
    tipos_de_dato::{comandos::hash_object::HashObject, logger, objeto::Objeto},
    utilidades_de_compresion::descomprimir_objeto,
};

use super::blob::Blob;

#[derive(Clone, Debug, PartialEq, Eq)]

pub struct Tree {
    pub directorio: PathBuf,
    pub objetos: Vec<Objeto>,
}
impl Tree {
    fn obtener_hash_completo(hash_20: String) -> Result<String, String> {
        let dir_candidatos = format!(".gir/objects/{}/", &hash_20[..2]);
        let candidatos = match fs::read_dir(&dir_candidatos) {
            Ok(candidatos) => candidatos,
            Err(_) => return Err(format!("No se econtro el objeto con hash {}", hash_20)),
        };

        for candidato in candidatos {
            let candidato_string = candidato.unwrap().path().display().to_string();
            let candidato_hash = candidato_string.split('/').last().unwrap().to_string();
            let (prefijo, hash_a_comparar) = hash_20.split_at(2);
            if candidato_hash.starts_with(&hash_a_comparar) {
                return Ok(format!("{}{}", prefijo, candidato_hash));
            }
        }

        Err(format!("No se econtro el objeto con hash {}", hash_20))
    }

    pub fn obtener_paths_hijos(&self) -> Vec<PathBuf> {
        let mut paths: Vec<PathBuf> = Vec::new();
        for objeto in &self.objetos {
            match objeto {
                Objeto::Blob(blob) => paths.push(blob.ubicacion.clone()),
                Objeto::Tree(tree) => {
                    paths.push(tree.directorio.clone());
                    paths.extend(tree.obtener_paths_hijos());
                }
            }
        }
        paths
    }

    fn obtener_datos_de_contenido(
        contenido: String,
    ) -> Result<Vec<(String, String, String)>, String> {
        let mut contenido_parseado: Vec<(String, String, String)> = Vec::new();
        let mut lineas = contenido.split("\0").collect::<Vec<&str>>();
        lineas.remove(0);

        for i in (0..lineas.len()).step_by(2) {
            if i + 1 < lineas.len() {
                let linea = lineas[i].split_whitespace();
                let linea_spliteada = linea.clone().collect::<Vec<&str>>();
                let modo = linea_spliteada[0];
                let nombre = linea_spliteada[1];
                let tupla = (
                    modo.to_string(),
                    nombre.to_string(),
                    lineas[i + 1].to_string(),
                );
                contenido_parseado.push(tupla);
            } else {
                return Err("Error al parsear el contenido del tree".to_string());
            }
        }
        Ok(contenido_parseado)
    }

    pub fn from_hash_20(hash: String, directorio: PathBuf) -> Tree {
        let hash_completo = Self::obtener_hash_completo(hash).unwrap();

        let contenido = descomprimir_objeto(hash_completo).unwrap();

        let contenido_parseado = Self::obtener_datos_de_contenido(contenido).unwrap();
        let mut objetos: Vec<Objeto> = Vec::new();

        for (modo, nombre, hash_hijo) in contenido_parseado {
            let ubicacion = format!("{}/{}", directorio.display(), nombre);

            match modo.as_str() {
                "100644" => {
                    let blob = Objeto::Blob(Blob {
                        nombre,
                        ubicacion: PathBuf::from(ubicacion),
                        hash: Self::obtener_hash_completo(hash_hijo.to_string()).unwrap(),
                    });
                    objetos.push(blob);
                }
                "40000" => {
                    let tree = Self::from_hash_20(hash_hijo, PathBuf::from(ubicacion));
                    objetos.push(Objeto::Tree(tree));
                }
                _ => {}
            }
        }

        Tree {
            directorio,
            objetos,
        }
    }

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

    pub fn agregar_hijo(&mut self, objeto: Objeto) {
        self.objetos.push(objeto);
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
        objetos.sort_by(|a, b| {
            let directorio_a = match a {
                Objeto::Blob(blob) => PathBuf::from(blob.nombre.clone()),
                Objeto::Tree(tree) => tree.directorio.clone(),
            };
            let directorio_b = match b {
                Objeto::Blob(blob) => PathBuf::from(blob.nombre.clone()),
                Objeto::Tree(tree) => tree.directorio.clone(),
            };
            directorio_a.cmp(&directorio_b)
        });
        objetos
    }

    pub fn escribir_en_base(&self) -> Result<(), String> {
        let hash = self.obtener_hash()?;
        let ruta = format!(".gir/objects/{}/{}", &hash[..2], &hash[2..]);

        let contenido = Self::mostrar_contenido(&self.objetos)?;
        let header = format!("tree {}\0", contenido.len());

        let contenido_total = format!("{}{}", header, contenido);
        io::escribir_bytes(&ruta, self.comprimir_contenido(contenido_total)?)?;

        for objeto in &self.objetos {
            match objeto {
                Objeto::Blob(blob) => {
                    let logger = Rc::new(logger::Logger::new(PathBuf::from("tmp/tree"))?);
                    HashObject {
                        logger: logger.clone(),
                        escribir: true,
                        ubicacion_archivo: blob.ubicacion.clone(),
                    }
                    .ejecutar()?;
                }
                Objeto::Tree(tree) => {
                    tree.escribir_en_base()?;
                }
            }
        }

        Ok(())
    }

    fn comprimir_contenido(&self, contenido: String) -> Result<Vec<u8>, String> {
        let mut compresor = ZlibEncoder::new(Vec::new(), Compression::default());
        if compresor.write_all(contenido.as_bytes()).is_err() {
            return Err("No se pudo comprimir el contenido".to_string());
        };
        match compresor.finish() {
            Ok(contenido_comprimido) => Ok(contenido_comprimido),
            Err(_) => Err("No se pudo comprimir el contenido".to_string()),
        }
    }

    fn mostrar_contenido(objetos: &Vec<Objeto>) -> Result<String, String> {
        let mut output = String::new();

        let objetos_ordenados = Self::ordenar_objetos_alfabeticamente(objetos);

        for objeto in objetos_ordenados {
            let line = match objeto {
                Objeto::Blob(blob) => format!("100644 {}\0{}", blob.nombre, &blob.hash[..20]),
                Objeto::Tree(tree) => {
                    let name = match tree.directorio.file_name() {
                        Some(name) => name,
                        None => return Err("Error al obtener el nombre del directorio".to_string()),
                    };
                    let hash = tree.obtener_hash()?;
                    format!("40000 {}\0{}", name.to_string_lossy(), &hash[..20])
                }
            };
            output.push_str(&line);
        }
        Ok(output)
    }
}

impl Display for Tree {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self.directorio.file_name() {
            Some(name) => name,
            None => return Err(std::fmt::Error),
        };
        let hash = match self.obtener_hash() {
            Ok(hash) => hash,
            Err(_) => return Err(std::fmt::Error),
        };
        let string = format!("40000 {} {}\n", hash, name.to_string_lossy());
        write!(f, "{}", string)
    }
}

#[cfg(test)]

mod test {
    use flate2::read::ZlibDecoder;

    use crate::io;
    use crate::tipos_de_dato::{objeto::Objeto, objetos::tree::Tree};
    use std::io::Read;
    use std::path::PathBuf;

    #[test]
    fn test01_test_obtener_hash() {
        let objeto = Objeto::from_directorio(PathBuf::from("test_dir/objetos")).unwrap();
        let hash = objeto.obtener_hash();
        assert_eq!(hash, "bf902127ac66b999327fba07a9f4b7a50b87922a");
    }

    #[test]
    fn test02_test_obtener_tamanio() {
        let objeto = Objeto::from_directorio(PathBuf::from("test_dir/muchos_objetos")).unwrap();
        let tamanio = objeto.obtener_tamanio().unwrap();
        assert_eq!(tamanio, 83);
    }

    #[test]
    fn test03_test_mostrar_contenido() {
        let objeto = Objeto::from_directorio(PathBuf::from("test_dir/objetos")).unwrap();

        if let Objeto::Tree(tree) = objeto {
            let contenido = Tree::mostrar_contenido(&tree.objetos).unwrap();
            assert_eq!(contenido, "100644 archivo.txt\02b824e648965b94c6c6b");
        } else {
            assert!(false)
        }
    }

    #[test]
    fn test04_test_mostrar_contenido_recursivo() {
        let objeto = Objeto::from_directorio(PathBuf::from("test_dir/")).unwrap();

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

    #[test]
    fn test05_escribir_en_base() -> Result<(), String> {
        let objeto = Objeto::from_directorio(PathBuf::from("test_dir/objetos")).unwrap();
        if let Objeto::Tree(tree) = objeto {
            tree.escribir_en_base().unwrap();

            let contenido_leido = io::leer_bytes(
                &".gir/objects/bf/902127ac66b999327fba07a9f4b7a50b87922a".to_string(),
            )?;
            let mut descompresor = ZlibDecoder::new(contenido_leido.as_slice());
            let mut contenido_descomprimido = String::new();
            descompresor
                .read_to_string(&mut contenido_descomprimido)
                .unwrap();

            assert_eq!(
                contenido_descomprimido,
                "tree 39\0100644 archivo.txt\02b824e648965b94c6c6b"
            );

            Ok(())
        } else {
            assert!(false);
            Err("No se pudo leer el directorio".to_string())
        }
    }

    #[test]
    fn test06_escribir_en_base_con_anidados() -> Result<(), String> {
        let objeto = Objeto::from_directorio(PathBuf::from("test_dir")).unwrap();
        if let Objeto::Tree(tree) = objeto {
            tree.escribir_en_base().unwrap();

            let contenido_leido = io::leer_bytes(
                &".gir/objects/e0/46b641e27871e304ef4ce3fdf4382265d58354".to_string(),
            )
            .unwrap();
            let mut descompresor = ZlibDecoder::new(contenido_leido.as_slice());
            let mut contenido_descomprimido = String::new();
            descompresor
                .read_to_string(&mut contenido_descomprimido)
                .unwrap();

            assert_eq!(
                contenido_descomprimido,
                "tree 75\040000 muchos_objetos\0748ef9d5f9df6f40b07b40000 objetos\0bf902127ac66b999327f"
            );

            Ok(())
        } else {
            assert!(false);
            Err("No se pudo leer el directorio".to_string())
        }
    }
}

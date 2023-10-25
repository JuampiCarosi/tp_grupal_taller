use std::{
    env::consts::OS,
    ffi::OsStr,
    fmt::{Display, Write},
    fs,
    num::ParseIntError,
    path::PathBuf,
    rc::Rc,
};

use sha1::{Digest, Sha1};

use crate::{
    io,
    tipos_de_dato::{comandos::hash_object::HashObject, logger, objeto::Objeto},
    utilidades_de_compresion::{comprimir_contenido_u8, descomprimir_objeto},
    utilidades_path_buf::{esta_directorio_habilitado, obtener_nombre},
};

use super::blob::Blob;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]

pub struct Tree {
    pub directorio: PathBuf,
    pub objetos: Vec<Objeto>,
}
impl Tree {
    fn obtener_objetos_hoja(&self) -> Vec<Objeto> {
        let mut objetos: Vec<Objeto> = Vec::new();
        for objeto in &self.objetos {
            match objeto {
                Objeto::Blob(blob) => objetos.push(Objeto::Blob(blob.clone())),
                Objeto::Tree(tree) => {
                    objetos.extend(tree.obtener_objetos_hoja());
                }
            }
        }
        objetos
    }

    pub fn escribir_en_directorio(&self) -> Result<(), String> {
        let objetos = self.obtener_objetos_hoja();
        for objeto in objetos {
            match objeto {
                Objeto::Blob(blob) => {
                    let objeto = descomprimir_objeto(blob.hash)?;
                    let contenido = objeto.split('\0').collect::<Vec<&str>>()[1];
                    io::escribir_bytes(blob.ubicacion, contenido).unwrap();
                }
                Objeto::Tree(_) => Err("Llego a un tree pero no deberia")?,
            };
        }
        Ok(())
    }

    pub fn decode_hex(s: &str) -> Result<Vec<u8>, String> {
        match (0..s.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&s[i..i + 2], 16))
            .collect::<Result<Vec<u8>, ParseIntError>>()
        {
            Ok(hex) => Ok(hex),
            Err(_) => Err(format!("Error al decodificar el hash {}", s)),
        }
    }

    pub fn encode_hex(bytes: &[u8]) -> String {
        let mut s = String::with_capacity(bytes.len() * 2);
        for &b in bytes {
            write!(&mut s, "{:02x}", b).unwrap();
        }
        s
    }

    pub fn obtener_hash(&self) -> Result<String, String> {
        let contenido = Self::obtener_contenido(&self.objetos)?;
        let mut sha1 = Sha1::new();

        let header = format!("tree {}\0", contenido.len());
        sha1.update([header.as_bytes(), &contenido].concat());

        let hash = sha1.finalize();
        Ok(format!("{:x}", hash))
    }

    pub fn obtener_contenido(objetos: &[Objeto]) -> Result<Vec<u8>, String> {
        let objetos_ordenados = Self::ordenar_objetos_alfabeticamente(&objetos);

        let mut contenido: Vec<u8> = Vec::new();

        for objeto in objetos_ordenados {
            let mut line = match objeto {
                Objeto::Blob(ref blob) => {
                    let hash = Self::decode_hex(&blob.hash)?;
                    [b"100644 ", blob.nombre.as_bytes(), b"\0", &hash].concat()
                }
                Objeto::Tree(tree) => {
                    let nombre = if tree.directorio == PathBuf::from(".") {
                        String::from(".")
                    } else {
                        obtener_nombre(&tree.directorio.clone())?
                    };
                    let hash = &Self::decode_hex(&tree.obtener_hash()?)?;
                    [b"40000 ", nombre.as_bytes(), b"\0", &hash].concat()
                }
            };
            contenido.append(&mut line);
        }
        Ok(contenido)
    }

    fn obtener_hash_completo(hash_incompleto: String) -> Result<String, String> {
        let dir_candidatos = format!(".gir/objects/{}/", &hash_incompleto[..2]);
        let candidatos = match fs::read_dir(&dir_candidatos) {
            Ok(candidatos) => candidatos,
            Err(_) => {
                return Err(format!(
                    "No se econtro el objeto con hash {}",
                    hash_incompleto
                ))
            }
        };

        for candidato in candidatos {
            let candidato_string = candidato
                .map_err(|_| {
                    format!(
                        "Error al extraer el hash {} de la carpeta padre",
                        hash_incompleto
                    )
                })?
                .path()
                .display()
                .to_string();
            let candidato_hash = candidato_string
                .split('/')
                .last()
                .ok_or_else(|| {
                    format!(
                        "Error al extraer carpeta hijo del padre {}, existe dicho hash?",
                        hash_incompleto
                    )
                })?
                .to_string();
            let (prefijo, hash_a_comparar) = hash_incompleto.split_at(2);
            if candidato_hash.starts_with(hash_a_comparar) {
                return Ok(format!("{}{}", prefijo, candidato_hash));
            }
        }

        Err(format!(
            "No se econtro el objeto con hash {}",
            hash_incompleto
        ))
    }

    pub fn from_directorio(
        directorio: PathBuf,
        hijos_especificados: Option<&Vec<PathBuf>>,
    ) -> Result<Tree, String> {
        let mut objetos: Vec<Objeto> = Vec::new();

        // if directorio.starts_with("./") && directorio != PathBuf::from("./") {
        //     directorio = directorio.strip_prefix("./").unwrap().to_path_buf();
        // }

        // println!("directorio: {}", directorio.display());

        let entradas = match fs::read_dir(&directorio) {
            Ok(entradas) => entradas,
            Err(_) => Err(format!("Error al leer el directorio {directorio:#?}"))?,
        };

        for entrada in entradas {
            let entrada = entrada
                .map_err(|_| format!("Error al leer entrada el directorio {directorio:#?}"))?;
            let path = entrada.path();

            if path.ends_with(".DS_Store")
                || path.starts_with("./.target")
                || path.starts_with("./.gir")
                || path.starts_with("./.git")
                || path == PathBuf::from("./gir")
                || path == PathBuf::from("./git")
                || path == PathBuf::from("./target")
                || path == PathBuf::from("./diagrama.png")
            {
                continue;
            }

            if let Some(hijos_especificados) = &hijos_especificados {
                if !esta_directorio_habilitado(&path, hijos_especificados) {
                    continue;
                }
            }

            let objeto = match fs::metadata(&path) {
                Ok(_) => Objeto::from_directorio(path, hijos_especificados)?,
                Err(_) => Err("Error al leer el archivo".to_string())?,
            };
            objetos.push(objeto);
        }

        Ok(Tree {
            directorio,
            objetos,
        })
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
        let mut lineas = contenido.split('\0').collect::<Vec<&str>>();
        lineas.remove(0);

        let mut lineas_separadas: Vec<&str> = Vec::new();
        lineas_separadas.push(lineas[0].clone());
        let ultima_linea = lineas.pop().unwrap();
        lineas.iter().skip(1).for_each(|x| {
            let (hash, modo_y_nombre) = x.split_at(40);
            lineas_separadas.push(hash);
            lineas_separadas.push(modo_y_nombre);
        });
        lineas_separadas.push(ultima_linea);

        for i in (0..lineas_separadas.len()).step_by(2) {
            if i + 1 < lineas_separadas.len() {
                let linea = lineas_separadas[i].split_whitespace();
                let linea_spliteada = linea.clone().collect::<Vec<&str>>();
                let modo = linea_spliteada[0];
                let nombre = linea_spliteada[1];
                let tupla = (
                    modo.to_string(),
                    nombre.to_string(),
                    lineas_separadas[i + 1].to_string(),
                );
                contenido_parseado.push(tupla);
            } else {
                return Err("Error al parsear el contenido del tree".to_string());
            }
        }

        Ok(contenido_parseado)
    }

    pub fn from_hash(hash: String, directorio: PathBuf) -> Result<Tree, String> {
        // let hash_completo = Self::obtener_hash_completo(hash)?;

        let contenido = descomprimir_objeto(hash)?;

        let contenido_parseado = Self::obtener_datos_de_contenido(contenido)?;
        let mut objetos: Vec<Objeto> = Vec::new();

        for (modo, nombre, hash_hijo) in contenido_parseado {
            let mut ubicacion = format!("{}/{}", directorio.display(), nombre);

            if directorio == PathBuf::from(".") {
                ubicacion = nombre.clone()
            }

            match modo.as_str() {
                "100644" => {
                    let blob = Objeto::Blob(Blob {
                        nombre,
                        ubicacion: PathBuf::from(ubicacion),
                        hash: hash_hijo.to_string(),
                    });
                    objetos.push(blob);
                }
                "40000" => {
                    let tree = Self::from_hash(hash_hijo, PathBuf::from(ubicacion))?;
                    objetos.push(Objeto::Tree(tree));
                }
                _ => {}
            }
        }

        Ok(Tree {
            directorio,
            objetos,
        })
    }

    pub fn obtener_tamanio(&self) -> Result<usize, String> {
        let contenido = match Self::obtener_contenido(&self.objetos) {
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
    pub fn contiene_hijo_por_nombre(&self, nombre_hijo: PathBuf) -> bool {
        for objeto in &self.objetos {
            if objeto.obtener_path() == nombre_hijo {
                return true;
            }
        }
        false
    }

    pub fn agregar_hijo(&mut self, objeto: Objeto) {
        self.objetos.push(objeto);
    }

    pub fn actualizar_hijos(&mut self, objeto_a_actualizar: Objeto) {
        for objeto in &mut self.objetos {
            match objeto {
                Objeto::Tree(tree) => {
                    if tree.directorio == objeto_a_actualizar.obtener_path() {
                        *objeto = objeto_a_actualizar.clone();
                    } else {
                        tree.actualizar_hijos(objeto_a_actualizar.clone());
                    };
                }
                Objeto::Blob(blob) => {
                    if blob.ubicacion == objeto_a_actualizar.obtener_path() {
                        *objeto = objeto_a_actualizar.clone();
                    }
                }
            }
        }
    }

    pub fn eliminar_hijo_por_directorio(&mut self, directorio: &PathBuf) {
        let mut i = 0;
        while i < self.objetos.len() {
            match &mut self.objetos[i] {
                Objeto::Blob(blob) => {
                    if blob.ubicacion == *directorio {
                        self.objetos.remove(i);
                    } else {
                        i += 1;
                    }
                }
                Objeto::Tree(ref mut tree) => {
                    if tree.directorio == *directorio {
                        self.objetos.remove(i);
                    } else {
                        if directorio.starts_with(tree.directorio.clone()) {
                            tree.eliminar_hijo_por_directorio(directorio);
                        }
                        i += 1;
                    }
                }
            }
        }
    }

    pub fn obtener_hash_old(&self) -> Result<String, String> {
        let contenido = Self::mostrar_contenido(&self.objetos)?;
        let header = format!("tree {}\0", contenido.len());

        let contenido_total = format!("{}{}", header, contenido);

        let mut hasher = Sha1::new();
        hasher.update(contenido_total);
        let hash = hasher.finalize();
        Ok(format!("{:x}", hash))
    }

    pub fn ordenar_objetos_alfabeticamente(objetos: &[Objeto]) -> Vec<Objeto> {
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

        // if PathBuf::from(&ruta).exists() || self.es_vacio() {
        //     return Ok(());
        // }

        let contenido = Self::obtener_contenido(&self.objetos)?;

        let header = format!("tree {}\0", contenido.len());
        let contenido_completo = [header.as_bytes(), &contenido].concat();

        io::escribir_bytes(&ruta, comprimir_contenido_u8(&contenido_completo)?)?;

        for objeto in &self.objetos {
            match objeto {
                Objeto::Blob(blob) => {
                    let logger = Rc::new(logger::Logger::new(PathBuf::from("tmp/tree"))?);

                    // if PathBuf::from(&blob.ubicacion).is_file() {
                    //     continue;
                    // }

                    HashObject {
                        logger: logger.clone(),
                        escribir: true,
                        ubicacion_archivo: blob.ubicacion.clone(),
                    }
                    .ejecutar()?;
                }
                Objeto::Tree(tree) => tree.escribir_en_base()?,
            };
        }

        Ok(())
    }

    fn mostrar_contenido(objetos: &[Objeto]) -> Result<String, String> {
        let mut output = String::new();

        let objetos_ordenados = Self::ordenar_objetos_alfabeticamente(objetos);

        for objeto in objetos_ordenados {
            let line = match objeto {
                Objeto::Blob(blob) => format!("100644 {}    {}\n", blob.nombre, blob.hash),
                Objeto::Tree(tree) => {
                    let name = match tree.directorio.file_name() {
                        Some(name) => name,
                        None => return Err("Error al obtener el nombre del directorio".to_string()),
                    };
                    let hash = tree.obtener_hash()?;
                    format!("40000 {}    {}\n", name.to_string_lossy(), hash)
                }
            };
            output.push_str(&line);
        }
        Ok(output)
    }

    pub fn es_vacio(&self) -> bool {
        if self.objetos.len() == 0 {
            return true;
        }
        self.objetos.iter().all(|objeto| match objeto {
            Objeto::Blob(_) => false,
            Objeto::Tree(tree) => tree.es_vacio(),
        })
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

    use crate::io;
    use crate::tipos_de_dato::{objeto::Objeto, objetos::tree::Tree};
    use crate::utilidades_de_compresion::descomprimir_contenido_u8;
    use std::path::PathBuf;

    #[test]
    fn test01_test_obtener_hash() {
        let objeto = Objeto::from_directorio(PathBuf::from("test_dir/objetos"), None).unwrap();

        if let Objeto::Tree(ref tree) = objeto {
            tree.escribir_en_base().unwrap();
        }
        let hash = objeto.obtener_hash();
        assert_eq!(hash, "1442e275fd3a2e743f6bccf3b11ab27862157179");
    }

    #[test]
    fn test02_test_obtener_tamanio() {
        let objeto =
            Objeto::from_directorio(PathBuf::from("test_dir/muchos_objetos"), None).unwrap();
        let tamanio = objeto.obtener_tamanio().unwrap();
        assert_eq!(tamanio, 83);
    }

    #[test]
    fn test03_test_mostrar_contenido() {
        let objeto = Objeto::from_directorio(PathBuf::from("test_dir/objetos"), None).unwrap();

        if let Objeto::Tree(tree) = objeto {
            let contenido = Tree::mostrar_contenido(&tree.objetos).unwrap();
            assert_eq!(
                contenido,
                "100644 archivo.txt    2b824e648965b94c6c6b3dd0702feb91f699ed62\n"
            );
        } else {
            assert!(false)
        }
    }

    #[test]
    fn test04_test_mostrar_contenido_recursivo() {
        let objeto = Objeto::from_directorio(PathBuf::from("test_dir/"), None).unwrap();

        if let Objeto::Tree(tree) = objeto {
            let contenido = Tree::mostrar_contenido(&tree.objetos).unwrap();
            assert_eq!(
                contenido,
                "40000 muchos_objetos    896ca4eb090e033d16d4e9b1027216572ac3eaae\n40000 objetos    1442e275fd3a2e743f6bccf3b11ab27862157179\n"
            );
        } else {
            assert!(false)
        }
    }

    #[test]
    fn test05_escribir_en_base() -> Result<(), String> {
        let objeto = Objeto::from_directorio(PathBuf::from("test_dir/objetos"), None).unwrap();
        if let Objeto::Tree(tree) = objeto {
            tree.escribir_en_base().unwrap();

            let contenido_leido =
                io::leer_bytes(".gir/objects/14/42e275fd3a2e743f6bccf3b11ab27862157179")?;

            let contenido_descomprimido = descomprimir_contenido_u8(&contenido_leido).unwrap();

            let contenido_esperado = [
                b"tree 39\0100644 archivo.txt\0".to_vec(),
                Tree::decode_hex("2b824e648965b94c6c6b3dd0702feb91f699ed62").unwrap(),
            ]
            .concat();

            assert_eq!(contenido_descomprimido, contenido_esperado);

            Ok(())
        } else {
            assert!(false);
            Err("No se pudo leer el directorio".to_string())
        }
    }

    #[test]
    fn test06_escribir_en_base_con_anidados() -> Result<(), String> {
        let objeto = Objeto::from_directorio(PathBuf::from("test_dir"), None).unwrap();
        if let Objeto::Tree(tree) = objeto {
            tree.escribir_en_base().unwrap();

            let contenido_leido =
                io::leer_bytes(".gir/objects/d1/bd5884df89a9734e3b0a4e7721a4802d85cce8").unwrap();
            let contenido_descomprimido = descomprimir_contenido_u8(&contenido_leido).unwrap();

            let contenido_esperado = [
                b"tree 75\040000 muchos_objetos\0".to_vec(),
                Tree::decode_hex("896ca4eb090e033d16d4e9b1027216572ac3eaae").unwrap(),
                b"40000 objetos\0".to_vec(),
                Tree::decode_hex("1442e275fd3a2e743f6bccf3b11ab27862157179").unwrap(),
            ]
            .concat();

            assert_eq!(contenido_descomprimido, contenido_esperado);

            Ok(())
        } else {
            assert!(false);
            Err("No se pudo leer el directorio".to_string())
        }
    }
}

use std::collections::HashMap;
use std::path::PathBuf;

use sha1::{Digest, Sha1};

use crate::io;
use crate::tipos_de_dato::objeto::Objeto;
use crate::tipos_de_dato::utilidades_index::generar_objetos_raiz;
use crate::tipos_de_dato::{objetos::tree::Tree, utilidades_index};
use crate::utilidades_de_compresion;

pub fn conseguir_arbol_padre_from_ult_commit(hash_commit_padre: String) -> String {
    let contenido =
        utilidades_de_compresion::descomprimir_objeto(hash_commit_padre.clone(), String::from("/home/juani/git/objects")).unwrap();
    let lineas_sin_null = contenido.replace("\0", "\n");
    let lineas = lineas_sin_null.split("\n").collect::<Vec<&str>>();
    let arbol_commit = lineas[1];
    let lineas = arbol_commit.split(" ").collect::<Vec<&str>>();
    let arbol_commit = lineas[1];
    arbol_commit.to_string()
}

fn mergear_arboles(arbol_index: &[Objeto], arbol_padre: &[Objeto]) -> Vec<Objeto> {
    let mut arbol_mergeado: HashMap<PathBuf, Objeto> = HashMap::new();

    for objeto_padre in arbol_padre {
        arbol_mergeado.insert(objeto_padre.obtener_path(), objeto_padre.clone());
    }
    for objeto_index in arbol_index {
        arbol_mergeado.insert(objeto_index.obtener_path(), objeto_index.clone());
    }
    arbol_mergeado.values().cloned().collect::<Vec<Objeto>>()
}

pub fn crear_arbol_commit(commit_padre: Option<String>) -> Result<String, String> {
    let objetos = utilidades_index::leer_index()?;
    if objetos.is_empty() {
        return Err("No hay archivos trackeados para commitear".to_string());
    }

    let objetos_raiz = utilidades_index::generar_objetos_raiz(&objetos)?;
    let mut objetos_a_utilizar = objetos_raiz.clone();

    if let Some(hash) = commit_padre {
        let hash_arbol_padre = conseguir_arbol_padre_from_ult_commit(hash.clone());
        let arbol_padre = Tree::from_hash(hash_arbol_padre.clone(), PathBuf::from("./"))?;
        let objetos_arbol_nuevo_commit = mergear_arboles(&objetos_raiz, &arbol_padre.objetos);
        let objetos_raiz_con_padre = generar_objetos_raiz(&objetos_arbol_nuevo_commit)?;
        objetos_a_utilizar = objetos_raiz_con_padre
    }

    let contenido_arbol_commit = Tree::obtener_contenido(&objetos_a_utilizar)?;

    let header = format!("tree {}\0", contenido_arbol_commit.len());
    let mut sha1 = Sha1::new();
    let contenido_total = [header.as_bytes(), &contenido_arbol_commit].concat();
    sha1.update(&contenido_total);
    let hash_bytes = sha1.finalize();
    let hash = format!("{:x}", hash_bytes);
    let ruta = format!(".gir/objects/{}/{}", &hash[..2], &hash[2..]);
    io::escribir_bytes(
        &ruta,
        utilidades_de_compresion::comprimir_contenido_u8(&contenido_total)?,
    )?;
    Ok(hash)
}

#[cfg(test)]
mod test {
    use std::{path::PathBuf, rc::Rc};

    use crate::{
        io,
        tipos_de_dato::{comandos::add::Add, comandos::init::Init, logger::Logger},
        utilidades_de_compresion,
    };

    use super::crear_arbol_commit;

    // fn addear_archivos_y_comittear(args: Vec<String>, logger: Rc<Logger>) {
    //     let mut add = Add::from(args, logger.clone()).unwrap();
    //     add.ejecutar().unwrap();
    //     let commit =
    //         Commit::from(&mut vec!["-m".to_string(), "mensaje".to_string()], logger).unwrap();
    //     commit.ejecutar().unwrap();
    // }

    // fn conseguir_hash_padre(branch: String) -> String {
    //     let hash = std::fs::read_to_string(format!(".gir/refs/heads/{}", branch)).unwrap();
    //     let contenido = utilidades_de_compresion::descomprimir_objeto(hash.clone()).unwrap();
    //     let lineas = contenido.split("\n").collect::<Vec<&str>>();
    //     let hash_padre = lineas[2];
    //     hash_padre.to_string()
    // }

    // fn conseguir_arbol_commit(branch: String) -> String {
    //     let hash_hijo = std::fs::read_to_string(format!(".gir/refs/heads/{}", branch)).unwrap();
    //     let contenido_hijo =
    //         utilidades_de_compresion::descomprimir_objeto(hash_hijo.clone()).unwrap();
    //     let lineas = contenido_hijo.split("\n").collect::<Vec<&str>>();
    //     let arbol_commit = lineas[1];
    //     let lineas = arbol_commit.split(" ").collect::<Vec<&str>>();
    //     let arbol_commit = lineas[1];
    //     arbol_commit.to_string()
    // }

    fn limpiar_archivo_gir() {
        io::rm_directorio(".gir").unwrap();
        let logger = Rc::new(Logger::new(PathBuf::from("tmp/branch_init")).unwrap());
        let init = Init {
            path: "./.gir".to_string(),
            logger,
        };
        init.ejecutar().unwrap();
    }

    #[test]
    // fn test01_commiteo_dos_archivos_en_commits_distintos_y_el_ultimo_contiene_commit_ambos() {
    //     limpiar_archivo_gir();
    //     let logger = Rc::new(Logger::new(PathBuf::from("tmp/commit_test01")).unwrap());
    //     addear_archivos_y_comittear(vec!["test_file.txt".to_string()], logger.clone());
    //     addear_archivos_y_comittear(vec!["test_file2.txt".to_string()], logger.clone());

    //     let hash_padre = conseguir_hash_padre("master".to_string());
    //     let hash_arbol = conseguir_arbol_commit("master".to_string());
    //     let contenido_padre =
    //         super::utilidades_de_compresion::descomprimir_objeto(hash_padre).unwrap();
    //     let contenido_arbol =
    //         super::utilidades_de_compresion::descomprimir_objeto(hash_arbol).unwrap();
    //     let lineas_padre = contenido_padre.split("\n").collect::<Vec<&str>>();
    //     let lineas_arbol = contenido_arbol.split("\n").collect::<Vec<&str>>();
    //     println!("{:?}", lineas_padre);
    //     println!("{:?}", lineas_arbol);
    // }
    fn test01_se_escribe_arbol_con_un_hijo() {
        limpiar_archivo_gir();
        let logger = Rc::new(Logger::new(PathBuf::from("tmp/commit_test01")).unwrap());
        Add::from(vec!["test_file.txt".to_string()], logger)
            .unwrap()
            .ejecutar()
            .unwrap();

        let arbol_commit = crear_arbol_commit(None).unwrap();
        let contenido_commit = utilidades_de_compresion::descomprimir_objeto(arbol_commit, String::from(".git/{}/{}")).unwrap();

        assert_eq!(
            contenido_commit,
            "tree 41\0100644 test_file.txt\0678e12dc5c03a7cf6e9f64e688868962ab5d8b65"
        );
    }
}

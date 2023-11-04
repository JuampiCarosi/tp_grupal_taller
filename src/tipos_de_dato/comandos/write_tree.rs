use std::collections::HashMap;
use std::path::PathBuf;

use crate::tipos_de_dato::objeto::Objeto;
use crate::tipos_de_dato::objetos::tree::Tree;
use crate::utilidades_de_compresion;
use crate::utilidades_index::{generar_objetos_raiz, leer_index, ObjetoIndex};

/// Dado un hash de un commit y una ubicacion de donde buscar el objeto
/// devuelve el hash del arbol de ese commit
pub fn conseguir_arbol_from_hash_commit(hash_commit_padre: &str, dir: String) -> String {
    let contenido =
        utilidades_de_compresion::descomprimir_objeto(hash_commit_padre.to_string(), dir).unwrap();
    let lineas_sin_null = contenido.replace("\0", "\n");
    let lineas = lineas_sin_null.split("\n").collect::<Vec<&str>>();
    let arbol_commit = lineas[1];
    let lineas = arbol_commit.split(" ").collect::<Vec<&str>>();
    let arbol_commit = lineas[1];
    arbol_commit.to_string()
}
pub fn conseguir_arbol_padre_from_ult_commit(hash_commit_padre: String) -> String {
    let contenido = utilidades_de_compresion::descomprimir_objeto(
        hash_commit_padre.clone(),
        String::from(".gir/objects/"),
    )
    .unwrap();
    let lineas_sin_null = contenido.replace("\0", "\n");
    let lineas = lineas_sin_null.split("\n").collect::<Vec<&str>>();
    let arbol_commit = lineas[1];
    let lineas = arbol_commit.split(" ").collect::<Vec<&str>>();
    let arbol_commit = lineas[1];
    arbol_commit.to_string()
}

/// Devuelve el arbol mergeado entre el arbol padre y los cambios trackeados en el index
fn aplicar_index_a_arbol(arbol_index: &[ObjetoIndex], arbol_padre: &[Objeto]) -> Vec<ObjetoIndex> {
    let mut arbol_mergeado: HashMap<PathBuf, ObjetoIndex> = HashMap::new();

    for objeto_padre in arbol_padre {
        let objeto_index = ObjetoIndex {
            es_eliminado: false,
            merge: false,
            objeto: objeto_padre.clone(),
        };
        arbol_mergeado.insert(objeto_padre.obtener_path(), objeto_index);
    }
    for objeto_index in arbol_index {
        if objeto_index.es_eliminado {
            arbol_mergeado.remove(&objeto_index.objeto.obtener_path());
            continue;
        }
        arbol_mergeado.insert(objeto_index.objeto.obtener_path(), objeto_index.clone());
    }
    arbol_mergeado
        .values()
        .cloned()
        .collect::<Vec<ObjetoIndex>>()
}

/// Crea un arbol de commit a partir del index y su commit padre
/// commit_padre es un option ya que puede ser No
/// ne en caso de que sea el primer commit
pub fn crear_arbol_commit(commit_padre: Option<String>) -> Result<String, String> {
    let objetos_index = leer_index()?;
    if objetos_index.is_empty() {
        return Err("No hay archivos trackeados para commitear".to_string());
    }

    let objetos_a_utilizar = if let Some(hash) = commit_padre {
        let hash_arbol_padre =
            conseguir_arbol_from_hash_commit(&hash, String::from(".gir/objects/"));
        let arbol_padre = Tree::from_hash(hash_arbol_padre.clone(), PathBuf::from("./"))?;
        let objetos_arbol_nuevo_commit =
            aplicar_index_a_arbol(&objetos_index, &arbol_padre.objetos);
        generar_objetos_raiz(&objetos_arbol_nuevo_commit)?
    } else {
        generar_objetos_raiz(&objetos_index)?
    };

    let arbol_commit = Tree {
        directorio: PathBuf::from("./"),
        objetos: objetos_a_utilizar,
    };

    arbol_commit.escribir_en_base()?;
    Ok(arbol_commit.obtener_hash()?)
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

    fn test01_se_escribe_arbol_con_un_hijo() {
        limpiar_archivo_gir();
        let logger = Rc::new(Logger::new(PathBuf::from("tmp/commit_test01")).unwrap());
        Add::from(vec!["test_file.txt".to_string()], logger)
            .unwrap()
            .ejecutar()
            .unwrap();

        let arbol_commit = crear_arbol_commit(None).unwrap();
        let contenido_commit = utilidades_de_compresion::descomprimir_objeto(
            arbol_commit,
            String::from(".gir/objects/"),
        )
        .unwrap();

        assert_eq!(
            contenido_commit,
            "tree 41\0100644 test_file.txt\0678e12dc5c03a7cf6e9f64e688868962ab5d8b65"
        );
    }

    #[test]
    fn test02_se_escribe_arbol_con_carpeta() {
        limpiar_archivo_gir();
        let logger = Rc::new(Logger::new(PathBuf::from("tmp/commit_test01")).unwrap());
        Add::from(vec!["test_dir/objetos".to_string()], logger)
            .unwrap()
            .ejecutar()
            .unwrap();

        let arbol_commit = crear_arbol_commit(None).unwrap();

        assert_eq!(arbol_commit, "01c6c27fe31e9a4c3e64d3ab3489a2d3716a2b49");

        let contenido_commit = utilidades_de_compresion::descomprimir_objeto(
            arbol_commit,
            ".gir/objects/".to_string(),
        )
        .unwrap();

        assert_eq!(
            contenido_commit,
            "tree 35\040000 test_dir\01f67151c34d6b33ec1a98fdafef8b021068395a0"
        );
    }
}

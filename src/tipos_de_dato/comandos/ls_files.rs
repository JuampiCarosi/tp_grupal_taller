use std::{path::PathBuf, sync::Arc};

use crate::{
    tipos_de_dato::{logger::Logger, objeto::Objeto, objetos::tree::Tree},
    utils::index::{leer_index, ObjetoIndex},
};

use super::status::obtener_arbol_del_commit_head;

pub struct LsFiles {
    logger: Arc<Logger>,
    trees_directorios: Vec<String>,
    index: Vec<ObjetoIndex>,
    archivos: Vec<String>,
    arbol_commit: Option<Tree>,
}

impl LsFiles {
    pub fn from(logger: Arc<Logger>, args: &mut Vec<String>) -> Result<LsFiles, String> {
        let mut trees_directorios = Vec::new();
        let mut archivos = Vec::new();

        let arbol_commit = obtener_arbol_del_commit_head(logger.clone());

        for arg in args {
            let path = PathBuf::from(arg.to_string());
            if !path.exists() {
                return Err(format!("No existe el archivo o directorio: {}", arg));
            }
            if path.is_dir() {
                trees_directorios.push(arg.to_string());
            } else {
                archivos.push(arg.to_string());
            }
        }

        let index = leer_index(logger.clone())?;

        Ok(LsFiles {
            logger,
            trees_directorios,
            index,
            archivos,
            arbol_commit,
        })
    }

    fn recorrer_arbol_hasta_hijo_buscado(
        direccion_hijo: &str,
        arbol: Tree,
    ) -> Result<Tree, String> {
        let path_hijo = PathBuf::from(direccion_hijo);
        for objeto in arbol.objetos {
            match objeto {
                Objeto::Tree(tree) => {
                    if tree.directorio == path_hijo {
                        return Ok(tree);
                    }
                }
                _ => continue,
            }
        }
        Err(format!(
            "No se encontro el directorio {} en el arbol",
            direccion_hijo
        ))
    }

    fn obtener_archivos_de_directorios(&self, arbol: Tree) -> Result<Vec<String>, String> {
        let mut texto_tree = Vec::new();
        if self.trees_directorios.is_empty() {
            let objetos_hoja = arbol.obtener_objetos_hoja();
            for objeto in objetos_hoja {
                texto_tree.push(format!("{}\n", objeto.obtener_path().display()));
            }
            return Ok(texto_tree);
        }
        for tree_directorio in &self.trees_directorios {
            let tree_buscado =
                Self::recorrer_arbol_hasta_hijo_buscado(tree_directorio, arbol.clone())?;
            let objetos_hoja = tree_buscado.obtener_objetos_hoja();
            for objeto in objetos_hoja {
                texto_tree.push(format!("{}\n", objeto.obtener_path().display()));
            }
        }
        Ok(texto_tree)
    }

    pub fn ejecutar(&self) -> Result<String, String> {
        self.logger.log("Ejecutando ls-files");
        let mut texto_a_mostrar = Vec::new();
        for archivo in &self.archivos {
            texto_a_mostrar.push(format!("{}\n", archivo));
        }
        if self.trees_directorios.is_empty() && !self.archivos.is_empty() {
            let string_final = texto_a_mostrar.concat();
            return Ok(string_final);
        }
        let mut texto_tree_e_index = Vec::new();
        match &self.arbol_commit {
            Some(arbol) => {
                let texto_tree = self.obtener_archivos_de_directorios(arbol.clone())?;
                texto_tree_e_index.extend(texto_tree);
            }
            None => {}
        }
        for objeto_index in &self.index {
            texto_tree_e_index.push(format!(
                "{}\n",
                objeto_index.objeto.obtener_path().display()
            ));
        }
        texto_tree_e_index.sort();
        texto_a_mostrar.extend(texto_tree_e_index);
        let string_final = texto_a_mostrar.concat();
        self.logger.log("Finalizando ls-files");
        Ok(string_final)
    }
}

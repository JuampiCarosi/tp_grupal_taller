use std::{
    path::{self, PathBuf},
    rc::Rc,
};

use crate::{
    io::leer_a_string,
    tipos_de_dato::{logger::Logger, objeto::Objeto, objetos::tree::Tree},
    utilidades_index::{leer_index, ObjetoIndex},
};

use super::{commit::Commit, write_tree::conseguir_arbol_padre_from_ult_commit};

pub struct Status {
    logger: Rc<Logger>,
    index: Vec<ObjetoIndex>,
    tree_commit_head: Option<Tree>,
    tree_directorio_actual: Tree,
}

pub fn obtener_arbol_del_commit_head() -> Option<Tree> {
    let ruta = match Commit::obtener_ruta_branch_commit() {
        Ok(ruta) => ruta,
        Err(_) => return None,
    };
    let padre_commit = leer_a_string(path::Path::new(&ruta)).unwrap_or_else(|_| "".to_string());
    if padre_commit == "" {
        None
    } else {
        let hash_arbol_commit = conseguir_arbol_padre_from_ult_commit(padre_commit);
        let tree = Tree::from_hash(hash_arbol_commit, PathBuf::from("./")).unwrap();
        Some(tree)
    }
}

impl Status {
    pub fn from(logger: Rc<Logger>) -> Result<Status, String> {
        let index = leer_index()?;
        let tree_commit_head = obtener_arbol_del_commit_head();
        let tree_directorio_actual = Tree::from_directorio(PathBuf::from("./"), None)?;
        Ok(Status {
            logger,
            index,
            tree_commit_head,
            tree_directorio_actual,
        })
    }

    pub fn obtener_staging(&self) -> Result<Vec<String>, String> {
        let mut staging = Vec::new();
        for objeto_index in &self.index {
            match self.tree_commit_head {
                Some(ref tree) => {
                    let tipo_cambio =
                        if tree.contiene_hijo_por_ubicacion(objeto_index.objeto.obtener_path()) {
                            match objeto_index.es_eliminado {
                                true => "eliminado",
                                false => "modificado",
                            }
                        } else {
                            "nuevo archivo"
                        };
                    let linea_formateada = format!(
                        "{}: {}",
                        tipo_cambio,
                        objeto_index.objeto.obtener_path().display()
                    );
                    staging.push(linea_formateada);
                }
                None => {
                    let linea_formateada = format!(
                        "nuevo archivo: {}",
                        objeto_index.objeto.obtener_path().display()
                    );
                    staging.push(linea_formateada);
                    continue;
                }
            }
        }
        Ok(staging)
    }

    pub fn obtener_trackeados(&self) -> Result<Vec<String>, String> {
        let mut trackeados = Vec::new();
        let tree_head = match self.tree_commit_head {
            Some(ref tree) => tree,
            None => return Ok(trackeados),
        };
        for objeto in self.tree_directorio_actual.obtener_objetos_hoja() {
            if tree_head.contiene_hijo_por_ubicacion(objeto.obtener_path()) {
                if !tree_head
                    .contiene_misma_version_hijo(objeto.obtener_hash(), objeto.obtener_path())
                    && !self.index.iter().any(|objeto_index| {
                        objeto_index.objeto.obtener_hash() == objeto.obtener_hash()
                    })
                {
                    trackeados.push(format!("modificado: {}", objeto.obtener_path().display()));
                }
            }
        }
        Ok(trackeados)
    }

    fn obtener_hijos_untrackeados(
        &self,
        tree: &Tree,
        tree_head: &Tree,
    ) -> Result<Vec<String>, String> {
        let mut untrackeados = Vec::new();

        for objeto in tree.objetos.iter() {
            if self
                .index
                .iter()
                .any(|objeto_index| objeto_index.objeto.obtener_hash() == objeto.obtener_hash())
            {
                continue;
            }
            match objeto {
                Objeto::Blob(_) => {
                    if !tree_head.contiene_hijo_por_ubicacion(objeto.obtener_path()) {
                        untrackeados.push(format!("{}", objeto.obtener_path().display()));
                    }
                }
                Objeto::Tree(ref tree) => {
                    if !tree_head.contiene_directorio(objeto.obtener_path()) {
                        untrackeados.push(format!("{}/", objeto.obtener_path().display()));
                    } else {
                        let mut untrackeados_hijos =
                            self.obtener_hijos_untrackeados(&tree, &tree_head)?;
                        untrackeados.append(&mut untrackeados_hijos);
                    }
                }
            }
        }
        Ok(untrackeados)
    }

    pub fn obtener_untrackeados(&self) -> Result<Vec<String>, String> {
        let tree_head = match self.tree_commit_head {
            Some(ref tree) => tree,
            None => {
                let mut untrackeados = Vec::new();
                for objeto in self.tree_directorio_actual.objetos.iter() {
                    untrackeados.push(format!("{}", objeto.obtener_path().display()));
                }
                return Ok(untrackeados);
            }
        };

        let untrackeados =
            self.obtener_hijos_untrackeados(&self.tree_directorio_actual, &tree_head)?;

        Ok(untrackeados)
    }

    pub fn ejecutar(&mut self) -> Result<String, String> {
        let staging = self.obtener_staging()?;
        let trackeados = self.obtener_trackeados()?;
        let untrackeados = self.obtener_untrackeados()?;

        let codigo_color_rojo = "\x1B[31m";
        let codigo_color_verde = "\x1B[32m";
        let codigo_color_predeterminado = "\x1B[0m";

        let mut mensaje = String::new();
        mensaje.push_str("Cambios a ser commiteados:\n");
        for cambio in staging {
            mensaje.push_str(&format!(
                "         {}{}{}\n",
                codigo_color_verde, cambio, codigo_color_predeterminado
            ));
        }
        mensaje.push_str("\nCambios no en zona de preparacion:\n");
        for cambio in trackeados {
            mensaje.push_str(&format!(
                "         {}{}{}\n",
                codigo_color_rojo, cambio, codigo_color_predeterminado
            ));
        }
        mensaje.push_str("\nCambios no trackeados:\n");
        for cambio in untrackeados {
            mensaje.push_str(&format!(
                "         {}{}{}\n",
                codigo_color_rojo, cambio, codigo_color_predeterminado
            ));
        }
        self.logger.log("Status terminado".to_string());
        Ok(mensaje)
    }
}

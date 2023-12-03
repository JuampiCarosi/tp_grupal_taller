use std::{path::PathBuf, sync::Arc};

use crate::tipos_de_dato::{
    comando::Ejecutar, logger::Logger, objeto::Objeto, objetos::tree::Tree,
    visualizaciones::Visualizaciones,
};

use super::cat_file::CatFile;

pub struct LsTree {
    logger: Arc<Logger>,
    recursivo: bool,
    solo_arboles: bool,
    con_size: bool,
    arbol: String,
}

impl LsTree {
    pub fn new(logger: Arc<Logger>, args: &mut Vec<String>) -> Result<LsTree, String> {
        let hash_arbol = args.pop().ok_or("No se pudo obtener el hash del arbol")?;
        if hash_arbol.len() != 40 {
            return Err(format!("El hash del arbol no es valido: {}", hash_arbol));
        }
        let mut recursivo = false;
        let mut solo_arboles = false;
        let mut con_size = false;

        for arg in args {
            match arg.as_str() {
                "-r" => recursivo = true,
                "-d" => solo_arboles = true,
                "-l" => con_size = true,
                _ => {
                    return Err(format!("Argumento no valido: {}", arg));
                }
            }
        }
        Ok(LsTree {
            logger,
            recursivo,
            solo_arboles,
            con_size,
            arbol: hash_arbol,
        })
    }

    fn obtener_string_blob(blob: &Objeto) -> String {
        format!(
            "100644 blob {}    {}\n",
            blob.obtener_hash(),
            blob.obtener_path().display()
        )
    }

    fn obtener_objetos_a_mostrar(&self, arbol: &Tree) -> Vec<Objeto> {
        let mut objetos_a_mostrar = Vec::new();
        if self.recursivo && self.solo_arboles {
            let hijos_totales = arbol.obtener_objetos();
            for hijo in hijos_totales {
                if let Objeto::Tree(tree) = hijo {
                    objetos_a_mostrar.push(Objeto::Tree(tree));
                }
            }
        } else if self.recursivo {
            objetos_a_mostrar = arbol.obtener_objetos_hoja();
        } else if self.solo_arboles {
            let hijos_arbol = arbol.objetos.clone();
            for hijo in hijos_arbol {
                if let Objeto::Tree(tree) = hijo {
                    objetos_a_mostrar.push(Objeto::Tree(tree));
                }
            }
        } else {
            objetos_a_mostrar = arbol.objetos.clone();
        }
        Tree::ordenar_objetos_alfabeticamente(&objetos_a_mostrar)
    }
}

impl Ejecutar for LsTree {
    fn ejecutar(&mut self) -> Result<String, String> {
        self.logger.log("Corriendo ls-tree");
        let arbol = Tree::from_hash(&self.arbol, PathBuf::from("."), self.logger.clone())?;
        let objetos_a_mostrar = self.obtener_objetos_a_mostrar(&arbol);

        let mut string_resultante = String::new();
        for objeto in objetos_a_mostrar {
            match objeto {
                Objeto::Blob(ref blob) => {
                    if self.con_size {
                        let tamanio = CatFile {
                            hash_objeto: blob.obtener_hash(),
                            logger: self.logger.clone(),
                            visualizacion: Visualizaciones::Tamanio,
                        }
                        .ejecutar()?;

                        string_resultante.push_str(&format!(
                            "100644 blob {} {: >7}    {}\n",
                            blob.obtener_hash(),
                            tamanio,
                            blob.ubicacion.display()
                        ));
                    } else {
                        string_resultante.push_str(&Self::obtener_string_blob(&objeto));
                    }
                }
                Objeto::Tree(ref tree) => {
                    if self.con_size {
                        string_resultante.push_str(&format!(
                            "040000 tree {}       -    {}\n",
                            tree.obtener_hash()?,
                            tree.directorio.display()
                        ));
                    } else {
                        string_resultante.push_str(&format!(
                            "040000 tree {}    {}\n",
                            tree.obtener_hash()?,
                            tree.directorio.display()
                        ));
                    }
                }
            }
        }
        Ok(string_resultante)
    }
}

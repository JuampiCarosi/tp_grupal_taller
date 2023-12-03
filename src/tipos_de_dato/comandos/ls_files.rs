use std::{path::PathBuf, sync::Arc};

use crate::{
    tipos_de_dato::{logger::Logger, objetos::tree::Tree},
    utils::index::{leer_index, ObjetoIndex},
};

use super::status::obtener_arbol_del_commit_head;

pub struct LsFiles {
    /// Logger para imprimir los mensajes en el archivo log.
    logger: Arc<Logger>,
    /// Paths de los directorios solicitados a mostrar.
    trees_directorios: Vec<String>,
    /// Contiene los objetos que estan en zona de staging.
    index: Vec<ObjetoIndex>,
    /// Paths de los archivos solicitados a mostrar.
    archivos: Vec<String>,
    /// Arbol del commit HEAD.
    arbol_commit: Option<Tree>,
}

impl LsFiles {
    /// Devuelve un LsFiles con los paths de los archivos/directorios a mostrar.
    /// Ademas, se inicializa el index y el arbol del commit HEAD.
    pub fn from(logger: Arc<Logger>, args: &mut Vec<String>) -> Result<LsFiles, String> {
        let mut trees_directorios = Vec::new();
        let mut archivos = Vec::new();

        let arbol_commit = obtener_arbol_del_commit_head(logger.clone());

        for arg in args {
            let path = PathBuf::from(arg.to_string());
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

    /// Apartir del vector de string con la ubicaion de los directorios solicitados,
    /// devuelve un vector con todos los archivos que se encuentran en esos directorios.
    /// Si no se especifica ningun directorio, devuelve todos los archivos del tree HEAD.
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
                Tree::recorrer_arbol_hasta_sub_arbol_buscado(tree_directorio, arbol.clone())?;
            let objetos_hoja = tree_buscado.obtener_objetos_hoja();
            for objeto in objetos_hoja {
                texto_tree.push(format!("{}\n", objeto.obtener_path().display()));
            }
        }
        Ok(texto_tree)
    }

    /// Devuelve un texto con los archivos pasados por parametro que existen en el tree HEAD.
    fn obtener_archivos_pedidos_por_parametro(&self) -> Vec<String> {
        let mut texto_a_mostrar = Vec::new();
        for archivo in &self.archivos {
            let path = PathBuf::from(archivo.to_string());
            match self.arbol_commit.clone() {
                Some(arbol) => {
                    if arbol.contiene_hijo_por_ubicacion(path) {
                        texto_a_mostrar.push(format!("{}\n", archivo));
                    }
                }
                None => {}
            }
        }
        texto_a_mostrar
    }

    /// Devuelve el texto de los archivos de los trees solicitados o tree HEAD, en conjunto
    /// con los archivos en zona de staging. Los archivos se devuelven ordenados.
    fn obtener_archivos_trackeados_e_index(&self) -> Result<Vec<String>, String> {
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
        Ok(texto_tree_e_index)
    }

    /// Ejecuta el comando ls-files.
    pub fn ejecutar(&self) -> Result<String, String> {
        self.logger.log("Ejecutando ls-files");
        let mut texto_a_mostrar = Vec::new();
        if self.trees_directorios.is_empty() && !self.archivos.is_empty() && self.index.is_empty() {
            let string_final = texto_a_mostrar.concat();
            return Ok(string_final);
        }
        let texto_archivos_pedidos = self.obtener_archivos_pedidos_por_parametro();
        texto_a_mostrar.extend(texto_archivos_pedidos);

        let texto_tree_e_index = self.obtener_archivos_trackeados_e_index()?;
        texto_a_mostrar.extend(texto_tree_e_index);

        let string_final = texto_a_mostrar.concat();
        self.logger.log("Finalizando ls-files");
        Ok(string_final)
    }
}

#[cfg(test)]
mod test {
    use std::{path::PathBuf, sync::Arc};

    use crate::{
        tipos_de_dato::{
            comandos::{add::Add, commit::Commit, init::Init},
            logger::Logger,
        },
        utils::{index::limpiar_archivo_index, io},
    };

    use super::LsFiles;

    fn addear_archivos_y_comittear(args: Vec<String>, logger: Arc<Logger>) {
        let mut add = Add::from(args, logger.clone()).unwrap();
        add.ejecutar().unwrap();
        let commit =
            Commit::from(&mut vec!["-m".to_string(), "mensaje".to_string()], logger).unwrap();
        commit.ejecutar().unwrap();
    }

    fn limpiar_archivo_gir() {
        io::rm_directorio(".gir").unwrap();
        let logger = Arc::new(Logger::new(PathBuf::from("tmp/ls_files_init")).unwrap());
        let init = Init {
            path: "./.gir".to_string(),
            logger,
        };
        init.ejecutar().unwrap();
    }

    #[test]
    fn test01_ls_files_muestra_los_archivos_en_staging() {
        limpiar_archivo_index().unwrap();
        let logger = Arc::new(Logger::new(PathBuf::from("tmp/ls_files_test01")).unwrap());
        let mut args = vec!["test_dir/objetos/archivo.txt".to_string()];
        let mut add = Add::from(args.clone(), logger.clone()).unwrap();
        add.ejecutar().unwrap();
        let ls_files = LsFiles::from(logger.clone(), &mut args).unwrap();
        let resultado = ls_files.ejecutar().unwrap();
        assert_eq!(resultado, "test_dir/objetos/archivo.txt\n");
    }

    #[test]
    fn test02_ls_files_muestra_los_archivos_trackeados() {
        limpiar_archivo_gir();
        let logger = Arc::new(Logger::new(PathBuf::from("tmp/ls_files_test02")).unwrap());
        addear_archivos_y_comittear(vec!["test_dir".to_string()], logger.clone());
        let mut args = vec![];
        let ls_files = LsFiles::from(logger.clone(), &mut args).unwrap();
        let resultado = ls_files.ejecutar().unwrap();
        assert_eq!(resultado, "test_dir/muchos_objetos/archivo.txt\ntest_dir/muchos_objetos/archivo_copy.txt\ntest_dir/objetos/archivo.txt\n");
    }

    #[test]
    fn test03_ls_files_muestra_subdirectorios_pedidos() {
        let logger = Arc::new(Logger::new(PathBuf::from("tmp/ls_files_test03")).unwrap());
        let mut args = vec!["test_dir/muchos_objetos".to_string()];
        let ls_files = LsFiles::from(logger.clone(), &mut args).unwrap();
        let resultado = ls_files.ejecutar().unwrap();
        assert_eq!(
            resultado,
            "test_dir/muchos_objetos/archivo.txt\ntest_dir/muchos_objetos/archivo_copy.txt\n"
        );
    }

    #[test]
    fn test04_ls_files_con_archivo_inexistente_devuelve_string_vacio() {
        let logger = Arc::new(Logger::new(PathBuf::from("tmp/ls_files_test04")).unwrap());
        let mut args = vec!["archivo_inexistente".to_string()];
        let ls_files = LsFiles::from(logger.clone(), &mut args).unwrap();
        let resultado = ls_files.ejecutar().unwrap();
        assert_eq!(resultado, "");
    }

    #[test]
    fn test05_ls_files_de_un_archivo_no_trackeado_devuelve_string_vacio() {
        limpiar_archivo_gir();
        let logger = Arc::new(Logger::new(PathBuf::from("tmp/ls_files_test05")).unwrap());
        let mut args = vec!["test_dir/objetos/archivo.txt".to_string()];
        let ls_files = LsFiles::from(logger.clone(), &mut args).unwrap();
        let resultado = ls_files.ejecutar().unwrap();
        assert_eq!(resultado, "");
    }
}

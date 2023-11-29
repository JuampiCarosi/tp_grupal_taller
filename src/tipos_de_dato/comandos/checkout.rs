use std::{collections::HashMap, path::PathBuf, sync::Arc};

use crate::{
    tipos_de_dato::{
        comandos::branch::Branch,
        config::{Config, RamasInfo},
        logger::Logger,
        objeto::Objeto,
        objetos::tree::Tree,
    },
    utils::{self, io},
};

use super::{show_ref::ShowRef, write_tree::conseguir_arbol_from_hash_commit};

const PATH_HEAD: &str = "./.gir/HEAD";

pub struct Checkout {
    /// Si es true, se crea una nueva rama.
    crear_rama: bool,
    /// Nombre de la rama a cambiar.
    rama_a_cambiar: String,
    /// Logger para imprimir mensajes en un archivo log.
    logger: Arc<Logger>,
}

/// Tipo de rama que se puede cambiar

enum TipoRama {
    Local,
    /// El primer string representa la ruta del remote
    /// El segundo string representa el hash del commit al que apunta
    Remota(String, String),
}

impl Checkout {
    /// Verifica si hay flags en los argumentos.
    fn hay_flags(args: &Vec<String>) -> bool {
        args.len() != 1
    }

    /// Verifica si la cantidad de argumentos son validos para el comando checkout.
    fn verificar_argumentos(args: &Vec<String>) -> Result<(), String> {
        if args.len() > 2 {
            return Err(
                "Argumentos desconocidos.\ngir checkcout [-b] <nombre-rama-cambiar>".to_string(),
            );
        }
        Ok(())
    }

    /// Crea una instancia de Checkout setteada para crear la branch.
    /// Si no se puede crear devuelve un error.
    fn crearse_con_flags(args: Vec<String>, logger: Arc<Logger>) -> Result<Checkout, String> {
        match (args[0].as_str(), args[1].clone()) {
            ("-b", rama) => Ok(Checkout {
                crear_rama: true,
                rama_a_cambiar: rama,
                logger,
            }),
            _ => Err("Argumentos invalidos.\ngir chekcout [-b] <nombre-rama-cambiar>".to_string()),
        }
    }

    /// Crea la instancia de checkout pertinente a los argumentos enviados.
    pub fn from(args: Vec<String>, logger: Arc<Logger>) -> Result<Checkout, String> {
        Self::verificar_argumentos(&args)?;

        if Self::hay_flags(&args) {
            return Self::crearse_con_flags(args, logger);
        }

        Ok(Checkout {
            crear_rama: false,
            rama_a_cambiar: args[0].to_string(),
            logger,
        })
    }

    /// Devuelve un vector con los nombres de las ramas existentes en el repositorio.
    pub fn obtener_ramas() -> Result<Vec<String>, String> {
        let directorio = ".gir/refs/heads";
        let entradas = std::fs::read_dir(directorio)
            .map_err(|e| format!("No se pudo leer el directorio:{}\n {}", directorio, e))?;

        let mut output = Vec::new();

        for entrada in entradas {
            let entrada = entrada
                .map_err(|_| format!("Error al leer entrada el directorio {directorio:#?}"))?;

            let nombre = utils::path_buf::obtener_nombre(&entrada.path())?;
            output.push(nombre)
        }

        Ok(output)
    }

    pub fn obtener_ramas_remotas(&self) -> Result<HashMap<String, String>, String> {
        let show_ref = ShowRef::from(vec![], self.logger.clone())?;
        let ramas = show_ref.obtener_referencias(PathBuf::from(".gir/refs/remotes"))?;
        Ok(ramas)
    }
    /// Verifica si la rama a cambiar ya existe.
    fn verificar_si_la_rama_existe(&self) -> Result<TipoRama, String> {
        let ramas = Self::obtener_ramas()?;

        if ramas.contains(&self.rama_a_cambiar) {
            return Ok(TipoRama::Local);
        }

        // key: refs/remotes/<remote>/<branch>
        // value: <commit>
        let ramas_remotas = self.obtener_ramas_remotas()?;

        for (ruta, commit) in ramas_remotas {
            if ruta.ends_with(&self.rama_a_cambiar) {
                return Ok(TipoRama::Remota(ruta, commit));
            }
        }

        Err(format!("Fallo: No existe la rama {}", self.rama_a_cambiar))
    }

    /// Devuelve el nombre de la rama actual.
    /// O sea, la rama a la que apunta el archivo HEAD.
    fn conseguir_rama_actual(&self, contenidio_head: &str) -> Result<String, String> {
        let partes: Vec<&str> = contenidio_head.split('/').collect();
        let rama_actual = partes
            .last()
            .ok_or_else(|| "Fallo en la lectura de HEAD".to_string())?
            .trim();
        Ok(rama_actual.to_string())
    }

    /// Cambia la referencia de la rama en el archivo HEAD.
    fn cambiar_ref_en_head(&self) -> Result<(), String> {
        let contenido_head = io::leer_a_string(PATH_HEAD)?;

        let rama_actual = self.conseguir_rama_actual(&contenido_head)?;

        let nuevo_head = contenido_head.replace(&rama_actual, &self.rama_a_cambiar);

        io::escribir_bytes(PATH_HEAD, nuevo_head)?;

        Ok(())
    }

    fn crear_rama_desde_remote(&self, commit: &str) -> Result<(), String> {
        io::escribir_bytes(format!(".gir/refs/heads/{}", self.rama_a_cambiar), commit)
    }

    fn configurar_remoto_para_rama_actual(&self, ruta_remoto: &str) -> Result<(), String> {
        let mut config = Config::leer_config()?;
        let rama = RamasInfo {
            nombre: self.rama_a_cambiar.clone(),
            remote: ruta_remoto.split('/').last().unwrap().to_string(),
            merge: PathBuf::from(format!("refs/heads/{}", self.rama_a_cambiar)),
        };

        config.ramas.push(rama);
        config.guardar_config()?;

        Ok(())
    }

    fn cambiar_rama(&self) -> Result<String, String> {
        match self.verificar_si_la_rama_existe()? {
            TipoRama::Remota(ruta, commit) => {
                self.crear_rama_desde_remote(&commit)?;
                self.configurar_remoto_para_rama_actual(&ruta)?
            }
            TipoRama::Local => {}
        };

        self.cambiar_ref_en_head()?;
        let msg = format!("Se cambio la rama actual a {}", self.rama_a_cambiar);
        self.logger.log(&msg);

        Ok(msg)
    }

    /// Crea una nueva rama con el nombre especificado.
    fn crear_rama(&self) -> Result<(), String> {
        let msg_branch = Branch::from(&mut vec![self.rama_a_cambiar.clone()], self.logger.clone())?
            .ejecutar()?;
        println!("{}", msg_branch);
        Ok(())
    }

    /// Verifica que el index no tenga contenido antes de cambiarse rama.
    fn comprobar_que_no_haya_contenido_index(&self) -> Result<(), String> {
        if !utils::index::esta_vacio_el_index()? {
            Err("Fallo, tiene contendio sin guardar. Por favor, haga commit para no perder los cambios".to_string())
        } else {
            Ok(())
        }
    }

    /// Devuelve el arbol del ultimo commit de la rama actual.
    fn obtener_arbol_commit_actual(&self) -> Result<Tree, String> {
        let ref_actual = io::leer_a_string(PATH_HEAD)?;
        let rama_actual = self.conseguir_rama_actual(&ref_actual)?;
        let head_commit = io::leer_a_string(format!(".gir/refs/heads/{}", rama_actual))?;
        let hash_tree_padre = conseguir_arbol_from_hash_commit(&head_commit, ".gir/objects/")?;
        Tree::from_hash(&hash_tree_padre, PathBuf::from("."), self.logger.clone())
    }

    /// Ejecuta el comando checkout en su totalidad.
    /// Si se crea una nueva rama, se crea y se cambia a ella.
    /// Si se cambia de rama, se cambia y se actualiza el contenido.
    pub fn ejecutar(&self) -> Result<String, String> {
        self.comprobar_que_no_haya_contenido_index()?;

        if self.crear_rama {
            self.crear_rama()?;
            self.cambiar_rama()?;
            return Ok(format!("Cambiado a nueva rama {}", self.rama_a_cambiar));
        };

        if !self.crear_rama {
            let tree_viejo = self.obtener_arbol_commit_actual()?;
            self.cambiar_rama()?;
            let tree_futuro = self.obtener_arbol_commit_actual()?;

            let objetos_a_eliminar = Self::obtener_objetos_eliminados(&tree_viejo, &tree_futuro);
            self.eliminar_objetos(&objetos_a_eliminar)?;

            tree_futuro.escribir_en_directorio()?;
        };
        Ok(format!("Cambiado a rama {}", self.rama_a_cambiar))
    }

    /// Elimina los archivos correspondientes a cada objeto que no se encuentre en el arbol futuro.
    fn eliminar_objetos(&self, objetos: &Vec<Objeto>) -> Result<(), String> {
        for objeto in objetos {
            match objeto {
                Objeto::Blob(blob) => {
                    io::rm_directorio(blob.ubicacion.clone())?;
                }
                Objeto::Tree(tree) => {
                    io::rm_directorio(tree.directorio.clone())?;
                }
            }
        }
        Ok(())
    }

    /// Devuelve un vector con los objetos que estaban en el tree viejo pero no en el nuevo.
    /// O sea, los objetos que se eliminaron.
    fn obtener_objetos_eliminados(tree_viejo: &Tree, tree_nuevo: &Tree) -> Vec<Objeto> {
        let mut objetos_eliminados: Vec<Objeto> = Vec::new();

        for objeto_viejo in tree_viejo.objetos.iter() {
            match objeto_viejo {
                Objeto::Blob(blob) => {
                    if !tree_nuevo.contiene_misma_version_hijo(&blob.hash, &blob.ubicacion) {
                        objetos_eliminados.push(objeto_viejo.clone());
                    }
                }
                Objeto::Tree(tree) => {
                    let mut hijos_eliminados = Self::obtener_objetos_eliminados(tree, tree_nuevo);
                    objetos_eliminados.append(&mut hijos_eliminados);
                }
            }
        }

        objetos_eliminados
    }
}

#[cfg(test)]
mod tests {
    use std::{path::PathBuf, sync::Arc};

    use crate::{
        tipos_de_dato::{
            comandos::{add::Add, branch::Branch, commit::Commit, init::Init},
            logger::Logger,
            objeto::Objeto,
            objetos::{blob::Blob, tree::Tree},
        },
        utils::io,
    };

    use super::*;

    fn craer_archivo_config_default() {
        let home = std::env::var("HOME").unwrap();
        let config_path = format!("{home}/.girconfig");
        let contenido = "nombre = aaaa\nmail = bbbb\n".to_string();
        io::escribir_bytes(config_path, contenido).unwrap();
    }

    fn limpiar_archivo_gir() {
        io::rm_directorio(".gir").unwrap();
        let logger = Arc::new(Logger::new(PathBuf::from("tmp/branch_init")).unwrap());
        let init = Init {
            path: "./.gir".to_string(),
            logger,
        };
        init.ejecutar().unwrap();
        craer_archivo_config_default();
    }

    fn addear_archivos_y_comittear(args: Vec<String>, logger: Arc<Logger>) {
        let mut add = Add::from(args, logger.clone()).unwrap();
        add.ejecutar().unwrap();
        let commit =
            Commit::from(&mut vec!["-m".to_string(), "mensaje".to_string()], logger).unwrap();
        commit.ejecutar().unwrap();
    }

    #[test]
    fn test01_checkout_cambia_de_rama() {
        limpiar_archivo_gir();
        let logger = Arc::new(Logger::new(PathBuf::from("tmp/checkout_test02")).unwrap());
        let args = vec!["test_dir/objetos/archivo.txt".to_string()];
        addear_archivos_y_comittear(args, logger.clone());

        Branch::from(&mut vec!["una_rama".to_string()], logger.clone())
            .unwrap()
            .ejecutar()
            .unwrap();

        let checkout = Checkout::from(vec!["una_rama".to_string()], logger.clone()).unwrap();
        checkout.ejecutar().unwrap();

        let contenido_head = std::fs::read_to_string(".gir/HEAD").unwrap();
        assert_eq!(contenido_head, "ref: refs/heads/una_rama".to_string());
    }

    #[test]

    fn test02_checkout_crea_y_cambia_de_rama() {
        limpiar_archivo_gir();
        let logger = Arc::new(Logger::new(PathBuf::from("tmp/checkout_test02")).unwrap());
        let args = vec!["test_dir/objetos/archivo.txt".to_string()];
        addear_archivos_y_comittear(args, logger.clone());

        let checkout = Checkout::from(
            vec!["-b".to_string(), "una_rama".to_string()],
            logger.clone(),
        )
        .unwrap();
        checkout.ejecutar().unwrap();

        let contenido_head = std::fs::read_to_string(".gir/HEAD").unwrap();
        assert_eq!(contenido_head, "ref: refs/heads/una_rama".to_string());
    }

    #[test]
    fn test03_al_hacer_checkout_actualiza_contenido() {
        limpiar_archivo_gir();
        let logger = Arc::new(Logger::new(PathBuf::from("tmp/checkout_test03")).unwrap());
        io::escribir_bytes("tmp/checkout_test03_test", "contenido").unwrap();
        let args = vec!["tmp/checkout_test03_test".to_string()];
        addear_archivos_y_comittear(args, logger.clone());

        let checkout = Checkout::from(
            vec!["-b".to_string(), "una_rama".to_string()],
            logger.clone(),
        )
        .unwrap();
        checkout.ejecutar().unwrap();

        io::escribir_bytes("tmp/checkout_test03_test", "contenido 2").unwrap();
        let args = vec!["tmp/checkout_test03_test".to_string()];
        addear_archivos_y_comittear(args, logger.clone());

        let checkout = Checkout::from(vec!["master".to_string()], logger.clone()).unwrap();
        checkout.ejecutar().unwrap();

        let contenido_archivo = io::leer_a_string("tmp/checkout_test03_test").unwrap();

        assert_eq!(contenido_archivo, "contenido".to_string());
    }

    #[test]
    fn test04_al_hacer_checkout_se_eliminan_no_trackeados() {
        limpiar_archivo_gir();
        let logger = Arc::new(Logger::new(PathBuf::from("tmp/checkout_test03")).unwrap());
        io::escribir_bytes("tmp/checkout_test04_test", "contenido").unwrap();
        let args = vec!["tmp/checkout_test04_test".to_string()];
        addear_archivos_y_comittear(args, logger.clone());

        let checkout = Checkout::from(
            vec!["-b".to_string(), "una_rama".to_string()],
            logger.clone(),
        )
        .unwrap();
        checkout.ejecutar().unwrap();

        io::escribir_bytes("tmp/checkout_test04_test_2", "contenido 2").unwrap();
        let args = vec!["tmp/checkout_test04_test_2".to_string()];
        addear_archivos_y_comittear(args, logger.clone());

        let checkout = Checkout::from(vec!["master".to_string()], logger.clone()).unwrap();
        checkout.ejecutar().unwrap();

        assert!(!PathBuf::from("tmp/checkout_test04_test_2").exists());
        assert!(PathBuf::from("tmp/checkout_test04_test").exists());
    }

    fn tree_con_un_tree_y_un_objeto(logger: Arc<Logger>) -> Tree {
        let objeto_nieto = Objeto::Blob(Blob {
            hash: "hash_nieto".to_string(),
            ubicacion: PathBuf::from("./tree_hijo/nieto"),
            logger: logger.clone(),
            nombre: "nieto".to_string(),
        });
        let objeto_hijo = Objeto::Blob(Blob {
            hash: "hash_hijo".to_string(),
            ubicacion: PathBuf::from("./hijo"),
            logger: logger.clone(),
            nombre: "hijo".to_string(),
        });

        let un_tree_hijo = Objeto::Tree(Tree {
            directorio: PathBuf::from("./tree_hijo"),
            objetos: vec![objeto_nieto],
            logger: logger.clone(),
        });

        Tree {
            directorio: PathBuf::from("."),
            objetos: vec![un_tree_hijo, objeto_hijo],
            logger: logger.clone(),
        }
    }

    fn tree_con_un_objeto(logger: Arc<Logger>) -> Tree {
        let objeto_hijo = Objeto::Blob(Blob {
            hash: "hash_hijo".to_string(),
            ubicacion: PathBuf::from("./hijo"),
            logger: logger.clone(),
            nombre: "hijo".to_string(),
        });

        Tree {
            directorio: PathBuf::from("."),
            objetos: vec![objeto_hijo],
            logger: logger.clone(),
        }
    }

    #[test]
    fn test05_obtener_objetos_eliminados() {
        let logger = Arc::new(Logger::new(PathBuf::from("tmp/checkout_test04")).unwrap());

        let tree_viejo = tree_con_un_tree_y_un_objeto(logger.clone());
        let tree_nuevo = tree_con_un_objeto(logger.clone());

        let objetos_eliminados = Checkout::obtener_objetos_eliminados(&tree_viejo, &tree_nuevo);

        assert_eq!(objetos_eliminados.len(), 1);

        if let Objeto::Blob(blob) = &objetos_eliminados[0] {
            assert_eq!(blob.nombre, "nieto".to_string());
        } else {
            unreachable!();
        }
    }
}

use std::{collections::HashSet, path::PathBuf, rc::Rc};

use crate::{
    io,
    tipos_de_dato::{
        comandos::branch::Branch, logger::Logger, objeto::Objeto, objetos::tree::Tree,
        utilidades_index,
    },
    utilidades_path_buf,
};

use super::write_tree::conseguir_arbol_padre_from_ult_commit;

const PATH_HEAD: &str = "./.gir/HEAD";

pub struct Checkout {
    crear_rama: bool,
    rama_a_cambiar: String,
    logger: Rc<Logger>,
}

impl Checkout {
    fn hay_flags(args: &Vec<String>) -> bool {
        args.len() != 1
    }

    fn verificar_argumentos(args: &Vec<String>) -> Result<(), String> {
        if args.len() > 2 {
            return Err(
                "Argumentos desconocidos.\ngir checkcout [-b] <nombre-rama-cambiar>".to_string(),
            );
        }
        Ok(())
    }

    fn crearse_con_flags(args: Vec<String>, logger: Rc<Logger>) -> Result<Checkout, String> {
        match (args[0].as_str(), args[1].clone()) {
            ("-b", rama) => {
                return Ok(Checkout {
                    crear_rama: true,
                    rama_a_cambiar: rama,
                    logger,
                })
            }
            _ => Err("Argumentos invalidos.\ngir chekcout [-b] <nombre-rama-cambiar>".to_string()),
        }
    }

    pub fn from(args: Vec<String>, logger: Rc<Logger>) -> Result<Checkout, String> {
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

    fn obtener_ramas(&self) -> Result<Vec<String>, String> {
        let directorio = ".gir/refs/heads";
        let entradas = std::fs::read_dir(directorio)
            .map_err(|e| format!("No se pudo leer el directorio:{}\n {}", directorio, e))?;

        let mut output = Vec::new();

        for entrada in entradas {
            let entrada = entrada
                .map_err(|_| format!("Error al leer entrada el directorio {directorio:#?}"))?;

            let nombre = utilidades_path_buf::obtener_nombre(&entrada.path())?;
            output.push(nombre)
        }

        Ok(output)
    }

    fn verificar_si_la_rama_existe(&self) -> Result<(), String> {
        let ramas = self.obtener_ramas()?;
        for rama in ramas {
            if rama == self.rama_a_cambiar {
                return Ok(());
            }
        }
        Err(format!("Fallo: No existe la rama {}", self.rama_a_cambiar))
    }

    fn conseguir_rama_actual(&self, contenidio_head: String) -> Result<String, String> {
        let partes: Vec<&str> = contenidio_head.split('/').collect();
        let rama_actual = partes
            .last()
            .ok_or_else(|| "Fallo en la lectura de HEAD".to_string())?
            .trim();
        Ok(rama_actual.to_string())
    }
    fn cambiar_ref_en_head(&self) -> Result<(), String> {
        let contenido_head = io::leer_a_string(PATH_HEAD)?;

        let rama_actual = self.conseguir_rama_actual(contenido_head.clone())?;

        let nuevo_head = contenido_head.replace(&rama_actual, &self.rama_a_cambiar);

        io::escribir_bytes(PATH_HEAD, nuevo_head)?;

        Ok(())
    }
    fn cambiar_rama(&self) -> Result<String, String> {
        self.verificar_si_la_rama_existe()?;
        self.cambiar_ref_en_head()?;

        let msg = format!("Se cambio la rama actual a {}", self.rama_a_cambiar);
        self.logger.log(msg.clone());

        Ok(msg)
    }

    fn crear_rama(&self) -> Result<(), String> {
        let msg_branch = Branch::from(&mut vec![self.rama_a_cambiar.clone()], self.logger.clone())?
            .ejecutar()?;
        print!("{}", msg_branch);
        Ok(())
    }

    fn comprobar_que_no_haya_contenido_index(&self) -> Result<(), String> {
        if !utilidades_index::esta_vacio_el_index()? {
            Err("Fallo, tiene contendio sin guardar. Por favor, haga commit para no perder los cambios".to_string())
        } else {
            Ok(())
        }
    }
    //si hay contenido en el index no swich

    fn obtener_arbol_commit_actual(&self) -> Result<Tree, String> {
        let ref_actual = io::leer_a_string(PATH_HEAD)?;
        let rama_actual = self.conseguir_rama_actual(ref_actual)?;
        let head_commit = io::leer_a_string(format!(".gir/refs/heads/{}", rama_actual))?;
        let hash_tree_padre = conseguir_arbol_padre_from_ult_commit(head_commit);
        Ok(Tree::from_hash(hash_tree_padre, PathBuf::from("."))?)
    }

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

            let objetos_a_eliminar = self.obtener_objetos_eliminados(&tree_viejo, &tree_futuro);
            self.eliminar_objetos(&objetos_a_eliminar)?;

            tree_futuro.escribir_en_directorio()?;
        };
        Ok(format!("Cambiado a rama {}", self.rama_a_cambiar))
    }

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

    fn obtener_objetos_eliminados(&self, tree_viejo: &Tree, tree_nuevo: &Tree) -> Vec<Objeto> {
        let mut objetos_eliminados: Vec<Objeto> = Vec::new();

        for objeto_viejo in tree_viejo.objetos.iter() {
            match objeto_viejo {
                Objeto::Blob(blob) => {
                    if blob.ubicacion == objeto_viejo.obtener_path() {
                        objetos_eliminados.push(objeto_viejo.clone());
                    }
                }
                Objeto::Tree(tree) => {
                    if tree.directorio == objeto_viejo.obtener_path() {
                        objetos_eliminados.push(objeto_viejo.clone());
                    }
                    // let mut hijos_eliminados = self.obtener_objetos_eliminados(tree, tree_nuevo);
                    // objetos_eliminados.append(&mut hijos_eliminados);
                }
            }
        }

        // println!("viejo : {:#?}", tree_viejo);
        // println!("nuevo: {:#?}", tree_nuevo);

        println!("objetos eliminados: {:#?}", objetos_eliminados);
        objetos_eliminados
    }

    fn deep_changes_entre_arboles(&self, arbol1: &Tree, arbol2: &Tree) -> Result<Tree, String> {
        let mut hijos: Vec<Objeto> = Vec::new();

        if arbol1.directorio != arbol2.directorio {
            return Err(format!(
                "Los directorios de los arboles no coinciden: {} y {}",
                arbol1.directorio.display(),
                arbol2.directorio.display()
            ));
        };

        let mut hijos_2_sin_usar: HashSet<&Objeto> = HashSet::from_iter(arbol2.objetos.iter());

        for hijo in &arbol1.objetos {
            let mut hijo_encontrado = false;
            for hijo2 in &arbol2.objetos {
                match (hijo, hijo2) {
                    (Objeto::Blob(b1), Objeto::Blob(b2)) => {
                        if b1.ubicacion == b2.ubicacion {
                            hijo_encontrado = true;
                            if b1.obtener_hash() != b2.obtener_hash() {
                                hijos.push(Objeto::Blob(b2.clone()));
                                hijos_2_sin_usar.remove(hijo2);
                                hijo_encontrado = false;
                            }
                        }
                    }
                    (Objeto::Tree(t1), Objeto::Tree(t2)) => {
                        if t1.directorio == t2.directorio {
                            hijo_encontrado = true;
                            if t1.obtener_hash() != t2.obtener_hash() {
                                hijos.push(Objeto::Tree(self.deep_changes_entre_arboles(t1, t2)?));
                                hijos_2_sin_usar.remove(hijo2);
                                hijo_encontrado = false;
                            }
                        }
                    }
                    _ => {}
                }
            }
            if !hijo_encontrado {
                hijos.push(hijo.clone());
            }
        }

        // for hijo2 in hijos_2_sin_usar {
        //     hijos.push(hijo2.clone());
        // }

        Ok(Tree {
            directorio: arbol1.directorio.clone(),
            objetos: hijos,
        })
    }
}

#[cfg(test)]
mod test {
    use std::{fs::OpenOptions, path::PathBuf, rc::Rc};

    use crate::{
        io::{self, rm_directorio},
        tipos_de_dato::{
            comandos::{add::Add, branch::Branch, commit::Commit, init::Init},
            logger::Logger,
        },
        utilidades_de_compresion,
    };

    use super::Checkout;

    fn limpiar_archivo_gir() {
        rm_directorio(".gir").unwrap();
        let logger = Rc::new(Logger::new(PathBuf::from("tmp/branch_init")).unwrap());
        let init = Init {
            path: "./.gir".to_string(),
            logger,
        };
        init.ejecutar().unwrap();
    }

    fn conseguir_hash_padre(branch: String) -> String {
        let hash = std::fs::read_to_string(format!(".gir/refs/heads/{}", branch)).unwrap();
        let contenido = utilidades_de_compresion::descomprimir_objeto(hash.clone()).unwrap();
        let lineas_sin_null = contenido.replace("\0", "\n");
        let lineas = lineas_sin_null.split("\n").collect::<Vec<&str>>();
        let hash_padre = lineas[2];
        hash_padre.to_string()
    }

    fn conseguir_arbol_commit(branch: String) -> String {
        let hash_hijo = std::fs::read_to_string(format!(".gir/refs/heads/{}", branch)).unwrap();
        let contenido_hijo =
            utilidades_de_compresion::descomprimir_objeto(hash_hijo.clone()).unwrap();
        let lineas_sin_null = contenido_hijo.replace("\0", "\n");
        let lineas = lineas_sin_null.split("\n").collect::<Vec<&str>>();
        let hash_arbol = lineas[1];
        hash_arbol.to_string()
    }

    fn addear_archivos_y_comittear(args: Vec<String>, logger: Rc<Logger>) {
        let mut add = Add::from(args, logger.clone()).unwrap();
        add.ejecutar().unwrap();
        let commit =
            Commit::from(&mut vec!["-m".to_string(), "mensaje".to_string()], logger).unwrap();
        commit.ejecutar().unwrap();
    }

    #[test]
    fn test01_checkout_cambia_de_rama() {
        limpiar_archivo_gir();
        let logger = Rc::new(Logger::new(PathBuf::from("tmp/checkout_test02")).unwrap());
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
        let logger = Rc::new(Logger::new(PathBuf::from("tmp/checkout_test02")).unwrap());
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
        let logger = Rc::new(Logger::new(PathBuf::from("tmp/checkout_test03")).unwrap());
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
        let logger = Rc::new(Logger::new(PathBuf::from("tmp/checkout_test03")).unwrap());
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

        assert!(!PathBuf::from("tmp/checkout_test04_test_2").exists())
    }
}

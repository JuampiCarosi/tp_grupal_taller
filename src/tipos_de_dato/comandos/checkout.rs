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
        return Ok(());
        if !utilidades_index::esta_vacio_el_index() {
            Err("Fallo, tiene contendio sin guardar. Por favor, haga commit para no perder los cambios".to_string())
        } else {
            Ok(())
        }
    }
    //si hay contenido en el index no swich
    pub fn ejecutar(&self) -> Result<String, String> {
        self.comprobar_que_no_haya_contenido_index()?;

        if self.crear_rama {
            self.crear_rama()?;
            self.cambiar_rama()?;
            return Ok(format!("Cambiado a nueva rama {}", self.rama_a_cambiar));
        };

        self.cambiar_rama()?;
        if !self.crear_rama {
            let tree_actual = Tree::from_directorio(PathBuf::from("."), None)?;
            let head_commit =
                io::leer_a_string(format!(".gir/refs/heads/{}", self.rama_a_cambiar))?;
            let hash_tree_padre = conseguir_arbol_padre_from_ult_commit(head_commit);
            let tree_padre = Tree::from_hash(hash_tree_padre, PathBuf::from("."))?;

            let tree = self.deep_changes_entre_arboles(&tree_actual, &tree_padre)?;
            tree.escribir_en_directorio()?;
        };
        Ok(format!("Cambiado a rama {}", self.rama_a_cambiar))
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
                            }
                        }
                    }
                    (Objeto::Tree(t1), Objeto::Tree(t2)) => {
                        if t1.directorio == t2.directorio {
                            hijo_encontrado = true;
                            if t1.obtener_hash() != t2.obtener_hash() {
                                hijos.push(Objeto::Tree(self.deep_changes_entre_arboles(t1, t2)?));
                                hijos_2_sin_usar.remove(hijo2);
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

        for hijo2 in hijos_2_sin_usar {
            hijos.push(hijo2.clone());
        }

        Ok(Tree {
            directorio: arbol1.directorio.clone(),
            objetos: hijos,
        })
    }
}

use std::{path::PathBuf, rc::Rc};

use crate::{io, tipos_de_dato::logger::Logger, utilidades_path_buf::obtener_nombre};

use super::commit::Commit;

const VERDE: &str = "\x1B[32m";
const RESET: &str = "\x1B[0m";

pub struct Branch {
    pub mostrar: bool,
    pub rama_nueva: Option<String>,
    pub logger: Rc<Logger>,
}

impl Branch {
    pub fn from(args: &mut Vec<String>, logger: Rc<Logger>) -> Result<Branch, String> {
        if args.len() == 0 {
            return Ok(Branch {
                mostrar: true,
                rama_nueva: None,
                logger,
            });
        }

        if args.len() > 1 {
            return Err("Demasiados argumentos\ngir branch [<nombre-rama-nueva>]".to_string());
        }

        let arg = args
            .pop()
            .ok_or(format!("No se pudo obtener el argumento de {:?}", args))?;

        Ok(Branch {
            mostrar: false,
            rama_nueva: Some(arg.to_string()),
            logger,
        })
    }

    pub fn obtener_ramas() -> Result<Vec<String>, String> {
        let directorio = ".gir/refs/heads";
        let entradas = std::fs::read_dir(directorio)
            .map_err(|e| format!("No se pudo leer el directorio:{}\n {}", directorio, e))?;

        let mut ramas: Vec<String> = Vec::new();

        for entrada in entradas {
            let entrada = entrada
                .map_err(|_| format!("Error al leer entrada el directorio {directorio:#?}"))?;
            let nombre = obtener_nombre(&entrada.path())?;
            ramas.push(nombre);
        }
        Ok(ramas)
    }

    pub fn mostrar_ramas() -> Result<String, String> {
        let rama_actual = Commit::obtener_branch_actual()?;

        let mut output = String::new();

        for rama in Self::obtener_ramas()? {
            if rama == rama_actual {
                output.push_str(&format!("* {}{}{}\n", VERDE, rama, RESET));
            } else {
                output.push_str(&format!("  {}\n", rama));
            }
        }

        Ok(output)
    }

    pub fn obtener_commit_head() -> Result<String, String> {
        let direccion_head = ".gir/HEAD";
        let direccion_branch_actual = io::leer_a_string(direccion_head)?;
        let branch_actual = direccion_branch_actual
            .split('/')
            .last()
            .ok_or(format!("No se pudo obtener el hash del commit"))?;
        let hash_commit = io::leer_a_string(format!(".gir/refs/heads/{}", branch_actual))?;
        Ok(hash_commit.to_string())
    }

    fn crear_rama(&mut self) -> Result<String, String> {
        let rama_nueva = self
            .rama_nueva
            .take()
            .ok_or("No se pudo obtener el nombre de la rama")?;

        let direccion_rama_nueva = format!(".gir/refs/heads/{}", rama_nueva);

        if PathBuf::from(&direccion_rama_nueva).exists() {
            return Err(format!("La rama {} ya existe", rama_nueva));
        }
        let ultimo_commit = Self::obtener_commit_head()?;
        io::escribir_bytes(direccion_rama_nueva, ultimo_commit)?;
        Ok(format!("Se creó la rama {}", rama_nueva))
    }

    pub fn ejecutar(&mut self) -> Result<String, String> {
        if self.mostrar {
            return Ok(Self::mostrar_ramas()?);
        }
        Ok(self.crear_rama()?)
    }
}

// #[cfg(test)]
// mod tests{
//     use super::Branch;
//     use crate::io::rm_directorio;
//     use crate::tipos_de_dato::comandos::add::Add;
//     use crate::tipos_de_dato::comandos::commit::Commit;
//     use crate::tipos_de_dato::comandos::init::Init;
//     use crate::tipos_de_dato::logger::Logger;
//     use crate::utilidades_de_compresion;
//     use std::path::PathBuf;
//     use std::rc::Rc;

//     fn craer_archivo_config_default() {
//         let config_path = "~/.girconfig";
//         let contenido = format!("nombre =aaaa\nmail =bbbb\n");
//         std::fs::write(config_path, contenido).unwrap();
//     }

//     fn limpiar_archivo_gir() {
//         if PathBuf::from("./.gir").exists() {
//             rm_directorio(".gir").unwrap();
//         }

//         let logger = Rc::new(Logger::new(PathBuf::from("tmp/branch_init")).unwrap());
//         let init = Init {
//             path: "./.gir".to_string(),
//             logger,
//         };
//         init.ejecutar().unwrap();
//         craer_archivo_config_default();
//     }

//     fn conseguir_arbol_commit(branch: String) -> String {
//         let hash_hijo = std::fs::read_to_string(format!(".gir/refs/heads/{}", branch)).unwrap();
//         let contenido_hijo =
//             utilidades_de_compresion::descomprimir_objeto(hash_hijo.clone()).unwrap();
//         let lineas_sin_null = contenido_hijo.replace("\0", "\n");
//         let lineas = lineas_sin_null.split("\n").collect::<Vec<&str>>();
//         let arbol_commit = lineas[1];
//         let lineas = arbol_commit.split(" ").collect::<Vec<&str>>();
//         let arbol_commit = lineas[1];
//         arbol_commit.to_string()
//     }

//     fn addear_archivos_y_comittear(args: Vec<String>, logger: Rc<Logger>) {
//         let mut add = Add::from(args, logger.clone()).unwrap();
//         add.ejecutar().unwrap();
//         let commit =
//             Commit::from(&mut vec!["-m".to_string(), "mensaje".to_string()], logger).unwrap();
//         commit.ejecutar().unwrap();
//     }

//     #[test]
//     fn test01_mostrar_ramas() {
//         limpiar_archivo_gir();
//         let logger = Rc::new(Logger::new(PathBuf::from("tmp/branch_test01")).unwrap());
//         let mut branch = Branch {
//             mostrar: true,
//             rama_nueva: None,
//             logger,
//         };

//         let resultado = branch.ejecutar();

//         assert_eq!(resultado, Ok("master\n".to_string()));
//     }

//     #[test]
//     fn test02_crear_rama() {
//         limpiar_archivo_gir();
//         let logger = Rc::new(Logger::new(PathBuf::from("tmp/branch_test02")).unwrap());
//         let mut branch = Branch {
//             mostrar: false,
//             rama_nueva: Some("nueva_rama".to_string()),
//             logger,
//         };

//         let resultado = branch.ejecutar();

//         assert_eq!(resultado, Ok("Se creó la rama nueva_rama".to_string()));
//     }

//     #[test]
//     fn test03_crear_una_rama_y_mostrar_ramas() {
//         limpiar_archivo_gir();
//         let logger = Rc::new(Logger::new(PathBuf::from("tmp/branch_test03")).unwrap());
//         Branch {
//             mostrar: false,
//             rama_nueva: Some("nueva_rama".to_string()),
//             logger: logger.clone(),
//         }
//         .ejecutar()
//         .unwrap();

//         let resultado = Branch {
//             mostrar: true,
//             rama_nueva: None,
//             logger,
//         }
//         .ejecutar();

//         assert_eq!(resultado, Ok("master\nnueva_rama\n".to_string()));
//     }

//     #[test]
//     fn test05_mostrar_from() {
//         limpiar_archivo_gir();
//         let logger = Rc::new(Logger::new(PathBuf::from("tmp/branch_test05")).unwrap());
//         let mut branch = Branch::from(&mut vec![], logger).unwrap();

//         let resultado = branch.ejecutar();

//         assert_eq!(resultado, Ok("master\n".to_string()));
//     }

//     #[test]
//     fn test06_crear_from() {
//         limpiar_archivo_gir();
//         let logger = Rc::new(Logger::new(PathBuf::from("tmp/branch_test06")).unwrap());
//         let mut branch = Branch::from(&mut vec!["nueva_rama".to_string()], logger).unwrap();

//         let resultado = branch.ejecutar();

//         assert_eq!(resultado, Ok("Se creó la rama nueva_rama".to_string()));
//     }

//     #[test]
//     #[should_panic(expected = "Demasiados argumentos\\ngir branch [<nombre-rama-nueva>]")]
//     fn test07_muchos_argumentos() {
//         limpiar_archivo_gir();
//         let logger = Rc::new(Logger::new(PathBuf::from("tmp/branch_test07")).unwrap());
//         let mut branch = Branch::from(
//             &mut vec!["nueva_rama".to_string(), "otra_nueva_rama".to_string()],
//             logger,
//         )
//         .unwrap();

//         branch.ejecutar().unwrap();
//     }

//     #[test]
//     fn test08_la_branch_se_crea_apuntando_al_ultimo_commit() {
//         limpiar_archivo_gir();
//         let logger = Rc::new(Logger::new(PathBuf::from("tmp/branch_test08")).unwrap());
//         addear_archivos_y_comittear(vec!["test_file.txt".to_string()], logger.clone());
//         let mut branch = Branch {
//             mostrar: false,
//             rama_nueva: Some("nueva_rama".to_string()),
//             logger: logger.clone(),
//         };
//         branch.ejecutar().unwrap();

//         let hash_arbol = conseguir_arbol_commit("nueva_rama".to_string());
//         let hash_arbol_git = "ce0ef9a25817847d31d12df1295248d24d07b309";

//         assert_eq!(hash_arbol, hash_arbol_git);
//     }
// }

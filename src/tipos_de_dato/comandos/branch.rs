use std::{path::PathBuf, rc::Rc};

use crate::{tipos_de_dato::logger::Logger, utilidades_path_buf::obtener_nombre, io};

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

    pub fn mostrar_ramas() -> Result<String, String> {
        let directorio = ".gir/refs/heads";
        let entradas = std::fs::read_dir(directorio).map_err(|e|format!("No se pudo leer el directorio:{}\n {}", directorio,e))?;

        let mut output = String::new();

        for entrada in entradas {
            let entrada = entrada
                .map_err(|_| format!("Error al leer entrada el directorio {directorio:#?}"))?;
            let nombre = obtener_nombre(&entrada.path())?;
            output.push_str(&format!("{}\n", nombre));
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

#[cfg(test)]
mod test {
    use super::Branch;
    use crate::io::rm_directorio;
    use crate::tipos_de_dato::comandos::init::Init;
    use crate::tipos_de_dato::logger::Logger;
    use std::path::PathBuf;
    use std::rc::Rc;

    fn limpiar_archivo_gir() {
        if PathBuf::from("./.gir").exists(){
            rm_directorio(".gir").unwrap();
        }
        
        let logger = Rc::new(Logger::new(PathBuf::from("tmp/branch_init")).unwrap());
        let init = Init {
            path: "./.gir".to_string(),
            logger,
        };
        init.ejecutar().unwrap();
    }

    #[test]
    fn test01_mostrar_ramas() {
        limpiar_archivo_gir();
        let logger = Rc::new(Logger::new(PathBuf::from("tmp/branch_test01")).unwrap());
        let mut branch = Branch {
            mostrar: true,
            rama_nueva: None,
            logger,
        };

        let resultado = branch.ejecutar();

        assert_eq!(resultado, Ok("master\n".to_string()));
    }

    #[test]
    fn test02_crear_rama() {
        limpiar_archivo_gir();
        let logger = Rc::new(Logger::new(PathBuf::from("tmp/branch_test02")).unwrap());
        let mut branch = Branch {
            mostrar: false,
            rama_nueva: Some("nueva_rama".to_string()),
            logger,
        };

        let resultado = branch.ejecutar();

        assert_eq!(resultado, Ok("Se creó la rama nueva_rama".to_string()));
    }

    #[test]
    fn test03_crear_una_rama_y_mostrar_ramas() {
        limpiar_archivo_gir();
        let logger = Rc::new(Logger::new(PathBuf::from("tmp/branch_test03")).unwrap());
        Branch {
            mostrar: false,
            rama_nueva: Some("nueva_rama".to_string()),
            logger: logger.clone(),
        }
        .ejecutar()
        .unwrap();

        let resultado = Branch {
            mostrar: true,
            rama_nueva: None,
            logger,
        }
        .ejecutar();

        assert_eq!(resultado, Ok("master\nnueva_rama\n".to_string()));
    }

    #[test]
    fn test05_mostrar_from() {
        limpiar_archivo_gir();
        let logger = Rc::new(Logger::new(PathBuf::from("tmp/branch_test05")).unwrap());
        let mut branch = Branch::from(&mut vec![], logger).unwrap();

        let resultado = branch.ejecutar();

        assert_eq!(resultado, Ok("master\n".to_string()));
    }

    #[test]
    fn test06_crear_from() {
        limpiar_archivo_gir();
        let logger = Rc::new(Logger::new(PathBuf::from("tmp/branch_test06")).unwrap());
        let mut branch = Branch::from(&mut vec!["nueva_rama".to_string()], logger).unwrap();

        let resultado = branch.ejecutar();

        assert_eq!(resultado, Ok("Se creó la rama nueva_rama".to_string()));
    }

    #[test]
    #[should_panic(expected = "Demasiados argumentos\\ngir branch [<nombre-rama-nueva>]")]
    fn test07_muchos_argumentos() {
        limpiar_archivo_gir();
        let logger = Rc::new(Logger::new(PathBuf::from("tmp/branch_test06")).unwrap());
        let mut branch = Branch::from(
            &mut vec!["nueva_rama".to_string(), "otra_nueva_rama".to_string()],
            logger,
        )
        .unwrap();

        branch.ejecutar().unwrap();
    }

    #[test]
    #[ignore = "falta terminar el commit"]
    fn test08_la_branch_se_crea_apuntando_al_ultimo_commit() {
        
    }
}

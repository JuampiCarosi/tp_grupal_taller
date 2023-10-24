use std::{path::PathBuf, rc::Rc};

use crate::{tipos_de_dato::logger::Logger, utilidades_path_buf::obtener_nombre};

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

    fn mostrar_ramas(&self) -> Result<String, String> {
        let directorio = ".gir/refs/heads";
        let ramas = std::fs::read_dir(directorio);

        let entradas = match ramas {
            Ok(entradas) => entradas,
            Err(_) => Err(format!("No se pudo leer el directorio {}\n", directorio))?,
        };

        let mut output = String::new();

        for entrada in entradas {
            let entrada = entrada
                .map_err(|_| format!("Error al leer entrada el directorio {directorio:#?}"))?;
            let nombre = obtener_nombre(&entrada.path())?;
            output.push_str(&format!("{}\n", nombre));
        }

        Ok(output)
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

        let resultado = std::fs::File::create(direccion_rama_nueva);

        match resultado {
            Ok(_) => Ok(format!("Se creó la rama {}", rama_nueva)),
            Err(_) => Err(format!("No se pudo crear la rama {}", rama_nueva)),
        }
    }

    pub fn ejecutar(&mut self) -> Result<String, String> {
        if self.mostrar {
            return Ok(self.mostrar_ramas()?);
        }

        Ok(self.crear_rama()?)
    }
}

#[cfg(test)]
mod test {
    use super::Branch;
    use crate::io::{crear_archivo, rm_directorio};
    use crate::tipos_de_dato::logger::Logger;
    use std::path::PathBuf;
    use std::rc::Rc;

    fn limpiar_heads() {
        rm_directorio(".gir/refs/heads").unwrap();
        crear_archivo(".gir/refs/heads/master").unwrap()
    }

    #[test]
    fn test01_mostrar_ramas() {
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
        limpiar_heads();
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
        limpiar_heads();
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
        limpiar_heads();
        let logger = Rc::new(Logger::new(PathBuf::from("tmp/branch_test05")).unwrap());
        let mut branch = Branch::from(&mut vec![], logger).unwrap();

        let resultado = branch.ejecutar();

        assert_eq!(resultado, Ok("master\n".to_string()));
    }

    #[test]
    fn test06_crear_from() {
        limpiar_heads();
        let logger = Rc::new(Logger::new(PathBuf::from("tmp/branch_test06")).unwrap());
        let mut branch = Branch::from(&mut vec!["nueva_rama".to_string()], logger).unwrap();

        let resultado = branch.ejecutar();

        assert_eq!(resultado, Ok("Se creó la rama nueva_rama".to_string()));
    }

    #[test]
    #[should_panic(expected = "Demasiados argumentos\\ngir branch [<nombre-rama-nueva>]")]
    fn test07_muchos_argumentos() {
        limpiar_heads();
        let logger = Rc::new(Logger::new(PathBuf::from("tmp/branch_test06")).unwrap());
        let mut branch = Branch::from(
            &mut vec!["nueva_rama".to_string(), "otra_nueva_rama".to_string()],
            logger,
        )
        .unwrap();

        branch.ejecutar().unwrap();
    }
}

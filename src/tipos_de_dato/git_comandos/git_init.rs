use std::{fs, path::Path, rc::Rc};

use crate::tipos_de_dato::logger::Logger;

pub struct GitInit {
    path: String,
    logger: Rc<Logger>,
}

impl GitInit {
    pub fn validate_args(args: Vec<String>) -> Result<(), String> {
        if args.len() > 1 {
            return Err("Argumentos desconocidos\n gir init [<directory>]".to_string());
        }

        Ok(())
    }

    pub fn from(args: Vec<String>, logger: Rc<Logger>) -> Result<GitInit, String> {
        Self::validate_args(args.clone())?;

        Ok(GitInit {
            path: Self::obtener_path(args),
            logger,
        })
    }

    pub fn ejecutar(&self) -> Result<(), String> {
        self.crear_directorio_git()
            .map_err(|err| format!("{}", err))?;

        self.logger
            .log(format!("Directorio gir creado en {}", self.path));

        Ok(())
    }

    fn obtener_path(args: Vec<String>) -> String {
        if args.is_empty() {
            "./.gir".to_string()
        } else {
            format!("{}{}", args[0], "/.gir")
        }
    }

    fn crear_directorio_git(&self) -> Result<(), std::io::Error> {
        self.verificar_si_ya_esta_creado_directorio_git()?;

        fs::create_dir_all(self.path.clone())?;
        fs::create_dir_all(self.path.clone() + "/objects")?;
        fs::create_dir_all(self.path.clone() + "/refs/heads")?;
        fs::create_dir_all(self.path.clone() + "/refs/tags")?;

        Ok(())
    }

    fn verificar_si_ya_esta_creado_directorio_git(&self) -> Result<(), std::io::Error> {
        if Path::new(&self.path).exists() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::AlreadyExists,
                "Ya existe un repositorio en este directorio",
            ));
        }

        Ok(())
    }
}

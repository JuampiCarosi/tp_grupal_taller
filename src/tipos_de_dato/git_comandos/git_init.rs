use std::{fs, path::Path, rc::Rc};

use crate::tipos_de_dato::logger::Logger;

pub struct GitInit {
    path: String,
    logger: Rc<Logger>,
}

impl GitInit {
    pub fn validar_argumentos(args: Vec<String>) -> Result<(), String> {
        if args.len() > 1 {
            return Err("Argumentos desconocidos\n gir init [<directory>]".to_string());
        }

        Ok(())
    }

    pub fn from(args: Vec<String>, logger: Rc<Logger>) -> Result<GitInit, String> {
        Self::validar_argumentos(args.clone())?;

        Ok(GitInit {
            path: Self::obtener_path(args),
            logger,
        })
    }

    pub fn ejecutar(&self) -> Result<(), String> {
        self.crear_directorio_git()
            .map_err(|err| format!("{}", err))?;

        let mensaje = format!("Directorio gir creado en {}", self.path);

        self.logger.log(mensaje.clone());
        println!("{}", mensaje);

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
        if self.verificar_si_ya_esta_creado_directorio_git() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::AlreadyExists,
                "Ya existe un repositorio en este directorio",
            ));
        };

        fs::create_dir_all(self.path.clone())?;
        fs::create_dir_all(self.path.clone() + "/objects")?;
        fs::create_dir_all(self.path.clone() + "/refs/heads")?;
        fs::create_dir_all(self.path.clone() + "/refs/tags")?;

        Ok(())
    }

    fn verificar_si_ya_esta_creado_directorio_git(&self) -> bool {
        Path::new(&self.path).exists()
    }
}

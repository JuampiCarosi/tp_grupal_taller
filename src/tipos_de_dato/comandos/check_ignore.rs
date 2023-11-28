use std::sync::Arc;

use crate::{tipos_de_dato::logger::Logger, utils::io};

pub struct CheckIgnore {
    logger: Arc<Logger>,
    paths: Vec<String>,
}

impl CheckIgnore {
    pub fn from(args: Vec<String>, logger: Arc<Logger>) -> Result<Self, String> {
        if args.is_empty() {
            return Err("Ingrese la ruta del archivo buscado como parametro".to_string());
        }

        let paths = args;

        Ok(CheckIgnore { logger, paths })
    }

    pub fn run(&self) -> Result<String, String> {
        let archivos_ignorados = io::leer_a_string(".girignore")?;
        let archivos_ignorados_separados: Vec<&str> = archivos_ignorados.split('\n').collect();

        let mut archivos_encontrados = Vec::new();

        for path in &self.paths {
            if archivos_ignorados_separados.contains(&path.as_str()) {
                archivos_encontrados.push(path);
            }
        }
        Ok(archivos_encontrados.join("\n"))
    }
}

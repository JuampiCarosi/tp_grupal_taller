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

    pub fn ejecutar(&self) -> Result<String, String> {
        self.logger.log("Buscando archivos ignorados".to_string());
        let archivos_ignorados = match io::leer_a_string(".girignore") {
            Ok(archivos_ignorados) => archivos_ignorados,
            Err(_) => return Err("Porfavor cree el archivo .girignore.".to_string()),
        };
        let archivos_ignorados_separados: Vec<&str> = archivos_ignorados.split('\n').collect();

        let mut archivos_encontrados: Vec<String> = Vec::new();

        for path in &self.paths {
            if archivos_ignorados_separados.contains(&path.as_str()) {
                archivos_encontrados.push(path.to_string());
            }
        }

        // for path in &self.paths {
        //     for archivos_ignorados in &archivos_ignorados_separados {
        //         if path.contains(archivos_ignorados) {
        //             archivos_encontrados.push(path.to_string());
        //         }
        //     }
        // }
        self.logger.log("Check ignore finalizado".to_string());

        Ok(archivos_encontrados.join("\n"))
    }
}

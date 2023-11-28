use crate::tipos_de_dato::logger::Logger;
use crate::utils;

use std::path::PathBuf;
use std::sync::Arc;

use super::init::Init;
use super::pull::Pull;
use super::remote::Remote;

const GIR_CLONE: &str = "gir clone <ip:puerto/repositorio/>";
pub struct Clone {
    logger: Arc<Logger>,
    url: String,
}

impl Clone {
    pub fn from(args: &mut Vec<String>, logger: Arc<Logger>) -> Result<Clone, String> {
        Self::verificar_argumentos(&args)?;

        let url = args.remove(0);

        logger.log(&format!("Se creo clone con exito - url: {}", url));

        Ok(Clone { logger, url })
    }

    fn verificar_argumentos(args: &Vec<String>) -> Result<(), String> {
        if args.len() != 1 {
            return Err(format!(
                "Parametros desconocidos {}\n {}",
                args.join(" "),
                GIR_CLONE
            ));
        };
        Ok(())
    }

    pub fn ejecutar(&mut self) -> Result<String, String> {
        let (_, mut repositorio) = utils::strings::obtener_ip_puerto_y_repositorio(&self.url)?;
        repositorio = repositorio.replace("/", "");

        self.verificar_si_ya_existe_repositorio(&repositorio)?;

        utils::io::crear_carpeta(&repositorio)?;
        utils::io::cambiar_directorio(&repositorio)?;

        let resutado = self.crear_repositorio();
        utils::io::cambiar_directorio("..")?;

        if let Err(e) = resutado {
            utils::io::rm_directorio(repositorio)?;
            return Err(e);
        }

        let mensaje = "Clone ejecutado con exito".to_string();
        self.logger.log(&mensaje);
        Ok(mensaje)
    }

    fn verificar_si_ya_existe_repositorio(&self, repositorio: &String) -> Result<(), String> {
        if PathBuf::from(repositorio).exists() {
            //me fijo si tiene contenido
            if utils::io::leer_directorio(repositorio)?.count() > 0 {
                return Err(format!("Error el directorio {} no esta vacio", repositorio));
            }
        }

        Ok(())
    }
    fn crear_repositorio(&mut self) -> Result<(), String> {
        Init::from(Vec::new(), self.logger.clone())?.ejecutar()?;

        let remote_args = &mut vec!["add".to_string(), "origin".to_string(), self.url.clone()];
        Remote::from(remote_args, self.logger.clone())?.ejecutar()?;

        let pull_args = vec!["-u".to_string(), "origin".to_string(), "master".to_string()];
        Pull::from(pull_args, self.logger.clone())?.ejecutar()?;
        Ok(())
    }
}

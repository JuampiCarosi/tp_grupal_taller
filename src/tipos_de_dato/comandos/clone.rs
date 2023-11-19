use crate::tipos_de_dato::logger::Logger;
use crate::utils;

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
    pub fn from(args: Vec<String>, logger: Arc<Logger>) -> Result<Clone, String> {
        Self::verificar_argumentos(&args)?;

        let url = args[0];

        logger.log(format!("Se creo clone con exito - url: {}", url));

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
        let (_, repositorio) = utils::strings::obtener_ip_puerto_y_repositorio(&self.url)?;

        utils::io::crear_carpeta(repositorio)?;
        utils::io::cambiar_directorio(&repositorio)?;

        let resutado = self.crear_repositorio(repositorio);
        utils::io::cambiar_directorio("..")?;

        if let Err(e) = resutado {
            utils::io::rm_directorio(repositorio)?;
            return Err(e);
        }

        let mensaje = "Clone ejecutado con exito".to_string();
        self.logger.log(mensaje.clone());
        Ok(mensaje)
    }

    fn crear_repositorio(&mut self, repositorio: String) -> Result<(), String> {
        Init::from(Vec::new(), self.logger.clone())?.ejecutar()?;

        let remote_args = &mut vec!["add".to_string(), "origin".to_string(), self.url.clone()];
        Remote::from(remote_args, self.logger.clone())?.ejecutar()?;

        let pull_args = vec!["-u".to_string(), "origin".to_string(), "master".to_string()];
        Pull::from(pull_args, self.logger.clone())?.ejecutar()?;
        Ok(())
    }
}

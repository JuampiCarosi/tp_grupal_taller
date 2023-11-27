use crate::tipos_de_dato::logger::Logger;

use std::sync::Arc;

use super::init::Init;
use super::pull::Pull;

const GIR_CLONE: &str = "gir clone <ip:puerto/repositorio/>";
pub struct Clone {
    logger: Arc<Logger>,
    url: String,
}

impl Clone {
    pub fn from(logger: Arc<Logger>) -> Result<Clone, String> {
        //Self::verificar_argumentos(&args)?;

        //let url = args[0];
        let url = String::new();
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
        Init::from(Vec::new(), self.logger.clone())?.ejecutar()?;
        Pull::from(Vec::new(), self.logger.clone())?.ejecutar()?;

        let mensaje = "Clone ejecutado con exito".to_string();
        self.logger.log(mensaje.clone());
        Ok(mensaje)
    }
}

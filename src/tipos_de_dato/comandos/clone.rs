use crate::tipos_de_dato::comando::Ejecutar;
use crate::tipos_de_dato::logger::Logger;
use crate::utils;

use std::net::TcpStream;
use std::path::PathBuf;
use std::sync::Arc;

use super::fetch::Fetch;
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
        Self::verificar_argumentos(args)?;

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

    fn verificar_si_ya_existe_repositorio(&self, repositorio: &str) -> Result<(), String> {
        if PathBuf::from(repositorio).exists() {
            //me fijo si tiene contenido
            if utils::io::leer_directorio(&repositorio)?.count() > 0 {
                return Err(format!("Error el directorio {} no esta vacio", repositorio));
            }
        }

        Ok(())
    }

    pub fn obtener_rama_predeterminada() -> Result<String, String> {
        let ramas = utils::io::leer_directorio(".gir/refs/remotes/origin")?;
        let mut rama_predeterminada = String::new();

        for rama in ramas {
            let rama = rama.map_err(|e| e.to_string())?;
            let nombre_rama = utils::path_buf::obtener_nombre(&rama.path())?;
            if nombre_rama == "master" {
                return Ok(nombre_rama);
            }
            rama_predeterminada = nombre_rama;
        }

        Ok(rama_predeterminada)
    }

    fn crear_repositorio(&mut self) -> Result<(), String> {
        Init::from(Vec::new(), self.logger.clone())?.ejecutar()?;

        let remote_args = &mut vec!["add".to_string(), "origin".to_string(), self.url.clone()];
        Remote::from(remote_args, self.logger.clone())?.ejecutar()?;

        Fetch::<TcpStream>::new(vec!["origin".to_string()], self.logger.clone())?.ejecutar()?;
        let rama_predeterminada = Self::obtener_rama_predeterminada()?;
        let pull_args = vec!["-u".to_string(), "origin".to_string(), rama_predeterminada];
        Pull::from(pull_args, self.logger.clone())?.ejecutar()?;
        Ok(())
    }
}

impl Ejecutar for Clone {
    fn ejecutar(&mut self) -> Result<String, String> {
        let (_, mut repositorio) = utils::strings::obtener_ip_puerto_y_repositorio(&self.url)?;
        repositorio = repositorio.replace('/', "");

        self.verificar_si_ya_existe_repositorio(&repositorio)?;

        utils::io::crear_carpeta(&repositorio)?;
        utils::io::cambiar_directorio(&repositorio)?;

        let resutado = self.crear_repositorio();
        utils::io::cambiar_directorio("..")?;

        resutado?;

        let mensaje = "Clone ejecutado con exito".to_string();
        self.logger.log(&mensaje);
        Ok(mensaje)
    }
}

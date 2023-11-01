use std::{collections::HashMap, rc::Rc};

use crate::{
    io::leer_a_string,
    tipos_de_dato::{
        config::{Config, RemoteInfo},
        logger::Logger,
    },
};

enum ComandoRemote {
    Mostrar,
    Agregar,
    Eliminar,
    CambiarUrl,
    MostrarUrl,
}

pub struct Remote {
    comando: ComandoRemote,
    nombre: Option<String>,
    url: Option<String>,
    logger: Rc<Logger>,
}

const INPUT_ERROR: &str = "gir remote add [<nombre-remote>] [<url-remote>]\ngir remote delete [<nombre-remote>] [<url-remote>]\ngir remote set-url [<nombre-remote>] [<url-remote>]\ngir remote show-url [<nombre-remote>] [<url-remote>]";

impl Remote {
    pub fn from(args: &mut Vec<String>, logger: Rc<Logger>) -> Result<Remote, String> {
        if args.len() > 3 {
            return Err(format!("Demasiados argumentos\n{}", INPUT_ERROR));
        }

        if args.len() == 0 {
            return Ok(Remote {
                comando: ComandoRemote::Mostrar,
                logger,
                nombre: None,
                url: None,
            });
        }

        if args.len() == 2 {
            match args[0].as_str() {
                "show-url" => {
                    return Ok(Remote {
                        comando: ComandoRemote::MostrarUrl,
                        logger,
                        nombre: Some(args[1].clone()),
                        url: None,
                    })
                }
                "delete" => {
                    return Ok(Remote {
                        comando: ComandoRemote::Eliminar,
                        logger,
                        nombre: Some(args[1].clone()),
                        url: None,
                    })
                }
                _ => return Err(INPUT_ERROR.to_string()),
            }
        }

        if args.len() == 3 {
            match args[0].as_str() {
                "add" => {
                    return Ok(Remote {
                        comando: ComandoRemote::Agregar,
                        logger,
                        nombre: Some(args[1].clone()),
                        url: Some(args[2].clone()),
                    })
                }
                "set-url" => {
                    return Ok(Remote {
                        comando: ComandoRemote::CambiarUrl,
                        logger,
                        nombre: Some(args[1].clone()),
                        url: Some(args[2].clone()),
                    })
                }
                _ => return Err(INPUT_ERROR.to_string()),
            }
        };

        Err(INPUT_ERROR.to_string())
    }

    fn agregar(&self) -> Result<String, String> {
        let mut config = Config::leer_config()?;

        let remote = RemoteInfo {
            nombre: self.nombre.clone().unwrap(),
            url: self.url.clone().unwrap(),
        };

        let remote_encontrada = config.remotes.iter().find(|r| r.nombre == remote.nombre);

        if remote_encontrada.is_some() {
            return Err(format!("Ya existe un remote con ese nombre"));
        }

        config.remotes.push(remote);
        config.guardar_config()?;

        Ok(format!(
            "Se agrego el remote {}",
            self.nombre.clone().unwrap()
        ))
    }

    fn eliminar(&self) -> Result<String, String> {
        let mut config = Config::leer_config()?;

        let nombre = self
            .nombre
            .clone()
            .ok_or("No se especifico el nombre del remote")?;

        let indice = config
            .remotes
            .iter()
            .position(|r| r.nombre == nombre.clone());

        if indice.is_none() {
            return Err(format!("No existe un remote con ese nombre"));
        }

        config.remotes.remove(indice.unwrap());
        config.guardar_config()?;

        Ok(format!("Se elimino el remote {}", nombre))
    }

    fn cambiar_url(&self) -> Result<String, String> {
        let mut config = Config::leer_config()?;

        let nombre = self
            .nombre
            .clone()
            .ok_or("No se especifico el nombre del remote")?;

        let url = self
            .url
            .clone()
            .ok_or("No se especifico la url del remote")?;

        let indice_result = config.remotes.iter().position(|r| r.nombre == nombre);

        let indice = match indice_result {
            Some(indice) => indice,
            None => return Err(format!("No existe un remote con ese nombre")),
        };

        config.remotes[indice] = RemoteInfo {
            nombre: nombre.clone(),
            url: url.clone(),
        };
        config.guardar_config()?;

        Ok(format!("Se cambio la url del remote {} a {}", nombre, url))
    }

    fn mostrar_url(&self) -> Result<String, String> {
        let config = Config::leer_config()?;

        let nombre = self
            .nombre
            .clone()
            .ok_or("No se especifico el nombre del remote")?;

        let remote = config.remotes.iter().find(|r| r.nombre == nombre);

        if remote.is_none() {
            return Err(format!("No existe un remote con ese nombre"));
        }

        config.guardar_config()?;

        Ok(format!(
            "La url del remote {} es {}",
            nombre,
            remote.unwrap().url
        ))
    }

    fn mostrar(&self) -> Result<String, String> {
        Ok("TODO".to_string())
    }

    pub fn ejecutar(&mut self) -> Result<String, String> {
        match &self.comando {
            ComandoRemote::Mostrar => self.mostrar(),
            ComandoRemote::Agregar => self.agregar(),
            ComandoRemote::Eliminar => self.eliminar(),
            ComandoRemote::CambiarUrl => self.cambiar_url(),
            ComandoRemote::MostrarUrl => self.mostrar_url(),
        }
    }
}

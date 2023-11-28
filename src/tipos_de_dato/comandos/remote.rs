use std::sync::Arc;

use crate::tipos_de_dato::{
    config::{Config, RemoteInfo},
    logger::Logger,
};

use super::commit::Commit;

enum ComandoRemote {
    Mostrar,
    Agregar,
    Eliminar,
    CambiarUrl,
    MostrarUrl,
}

pub struct Remote {
    /// Comando a ejecutar.
    comando: ComandoRemote,
    /// Nombre del remote.
    nombre: Option<String>,
    /// Url del remote.
    url: Option<String>,
    /// Logger para imprimir mensajes en el archivo log.
    logger: Arc<Logger>,
}

const INPUT_ERROR: &str = "gir remote add [<nombre-remote>] [<url-remote>]\ngir remote delete [<nombre-remote>] [<url-remote>]\ngir remote set-url [<nombre-remote>] [<url-remote>]\ngir remote show-url [<nombre-remote>] [<url-remote>]";

impl Remote {
    /// Crea una instancia de Remote.
    /// Si la cantidad de argumentos es mayor a 3 devuelve error.
    /// Si la cantidad de argumentos es 0 devuelve una instancia de Remote con el comando Mostrar.
    /// Si la cantidad de argumentos es 2 devuelve una instancia de Remote con el comando Eliminar o MostrarUrl.
    /// Si la cantidad de argumentos es 3 devuelve una instancia de Remote con el comando Agregar o CambiarUrl.
    pub fn from(args: &mut Vec<String>, logger: Arc<Logger>) -> Result<Remote, String> {
        if args.len() > 3 {
            return Err(format!("Demasiados argumentos\n{}", INPUT_ERROR));
        }

        if args.is_empty() {
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

    /// Agrega un remote a la configuración.
    fn agregar(&self) -> Result<String, String> {
        let mut config = Config::leer_config()?;

        let remote = RemoteInfo {
            nombre: self.nombre.clone().unwrap(),
            url: self.url.clone().unwrap(),
        };

        let remote_encontrada = config.remotos.iter().find(|r| r.nombre == remote.nombre);

        if remote_encontrada.is_some() {
            return Err("Ya existe un remote con ese nombre".to_string());
        }

        config.remotos.push(remote);
        config.guardar_config()?;

        Ok(format!(
            "Se agrego el remote {}",
            self.nombre.clone().unwrap()
        ))
    }

    /// Elimina un remote de la configuración.
    fn eliminar(&self) -> Result<String, String> {
        let mut config = Config::leer_config()?;

        let nombre = self
            .nombre
            .clone()
            .ok_or("No se especifico el nombre del remote")?;

        let indice = config
            .remotos
            .iter()
            .position(|r| r.nombre == nombre.clone());

        if indice.is_none() {
            return Err("No existe un remote con ese nombre".to_string());
        }

        config.remotos.remove(indice.unwrap());
        config.guardar_config()?;

        Ok(format!("Se elimino el remote {}", nombre))
    }

    /// Cambia la url de un remote de la configuración.
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

        let indice_result = config.remotos.iter().position(|r| r.nombre == nombre);

        let indice = match indice_result {
            Some(indice) => indice,
            None => return Err("No existe un remote con ese nombre".to_string()),
        };

        config.remotos[indice] = RemoteInfo {
            nombre: nombre.clone(),
            url: url.clone(),
        };
        config.guardar_config()?;

        Ok(format!("Se cambio la url del remote {} a {}", nombre, url))
    }

    /// Muestra la url asociada a un remote de la configuración.
    fn mostrar_url(&self) -> Result<String, String> {
        let config = Config::leer_config()?;

        let nombre = self
            .nombre
            .clone()
            .ok_or("No se especifico el nombre del remote")?;

        let remote = config.remotos.iter().find(|r| r.nombre == nombre);

        if remote.is_none() {
            return Err("No existe un remote con ese nombre".to_string());
        }

        config.guardar_config()?;

        Ok(format!(
            "La url del remote {} es {}",
            nombre,
            remote.unwrap().url
        ))
    }

    /// Muestra el remote asociado a la rama actual.
    fn mostrar(&self) -> Result<String, String> {
        let config = Config::leer_config()?;

        let branch_actual = Commit::obtener_branch_actual()?;
        let remote_actual = config
            .ramas
            .iter()
            .find(|branch| branch.nombre == branch_actual);

        match remote_actual {
            Some(remote) => Ok(remote.remote.clone()),
            None => return Err("No hay un remote asociado a la branch actual\n".to_string()),
        }
    }

    /// Ejecuta el comando.
    pub fn ejecutar(&mut self) -> Result<String, String> {
        self.logger.log("Ejecutando comando remote".to_string());
        match &self.comando {
            ComandoRemote::Mostrar => self.mostrar(),
            ComandoRemote::Agregar => self.agregar(),
            ComandoRemote::Eliminar => self.eliminar(),
            ComandoRemote::CambiarUrl => self.cambiar_url(),
            ComandoRemote::MostrarUrl => self.mostrar_url(),
        }
    }
}

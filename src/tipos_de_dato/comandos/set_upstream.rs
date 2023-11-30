use std::{path::PathBuf, sync::Arc};

use crate::{
    tipos_de_dato::{
        config::{Config, RamasInfo},
        logger::Logger,
    },
    utils,
};

pub struct SetUpstream {
    remoto: String,
    rama_remota: String,
    rama_local: String,
    logger: Arc<Logger>,
}

impl SetUpstream {
    pub fn new(
        remoto: String,
        rama_remota: String,
        rama_local: String,
        logger: Arc<Logger>,
    ) -> Result<SetUpstream, String> {
        logger.log(&format!(
            "Se crea set-upstream - remoto: {}, rama remota: {},rama local: {}",
            remoto, rama_remota, rama_remota
        ));

        Ok(SetUpstream {
            remoto,
            rama_remota,
            rama_local,
            logger,
        })
    }
    pub fn ejecutar(&self) -> Result<(), String> {
        self.logger.log(&format!(
            "Se ejecuta set-upstream - remoto: {}, rama remota: {},rama local: {}",
            self.remoto, self.rama_remota, self.rama_remota
        ));

        self.verificar_remoto()?;
        self.verificar_rama_local()?;

        self.set_upstream()?;

        self.logger.log(&format!(
            "Se ejecuto set-upstream con exito - remoto: {}, rama remota: {},rama local: {}",
            self.remoto, self.rama_remota, self.rama_remota
        ));
        Ok(())
    }

    ///Setea la rama asosiadandola al remoto y seteando el campo de merge. Para ello escribie
    /// en el archivo config.
    /// En caso de que ya esta seteada, lo actualiza
    fn set_upstream(&self) -> Result<(), String> {
        let mut config = Config::leer_config()?;
        let merge = PathBuf::from(format!("refs/heads/{}", self.rama_remota));

        let nueva_config_rama = RamasInfo {
            nombre: self.rama_local.clone(),
            remote: self.remoto.clone(),
            merge: merge,
        };

        let indice_resultado = config
            .ramas
            .iter()
            .position(|r| r.nombre == self.rama_local);

        match indice_resultado {
            Some(indice) => config.ramas[indice] = nueva_config_rama,
            None => config.ramas.push(nueva_config_rama),
        }

        config.guardar_config()
    }

    fn verificar_remoto(&self) -> Result<(), String> {
        if let false = Config::leer_config()?.existe_remote(&self.remoto) {
            return Err(format!(
                "Remoto desconocido: {}\n No se puede usar set-upstream\n",
                self.remoto
            ));
        };

        Ok(())
    }

    fn verificar_rama_local(&self) -> Result<(), String> {
        if !utils::ramas::existe_la_rama(&self.rama_local) {
            return Err(format!(
                "Rama desconocida: {}\n No se puede usar set-upstream\n",
                self.rama_local
            ));
        }

        Ok(())
    }
}

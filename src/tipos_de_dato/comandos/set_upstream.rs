use std::sync::Arc;

use crate::{
    tipos_de_dato::{
        config::{self, Config},
        logger::{self, Logger},
    },
    utils,
};

use super::remote;

pub struct SetUpstream {
    remoto: String,
    rama_remota: String,
    rama_local: String,
    logger: Arc<Logger>,
}

const SET_UPSTREAM: &str = "SET_UPSTREAM: <remoto> <rama-remota> <rama-local>";

impl SetUpstream {
    pub fn new(
        remoto: String,
        rama_remota: String,
        rama_local: String,
        logger: Arc<Logger>,
    ) -> Result<SetUpstream, String> {
        self.logger.log(format!(
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
        self.logger.log(format!(
            "Se ejecuta set-upstream - remoto: {}, rama remota: {},rama local: {}",
            self.remoto, self.rama_remota, self.rama_remota
        ));

        self.verificar_remoto()?;
        self.verificar_rama_local()?;

        Ok(())
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
}

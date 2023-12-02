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
            self.remoto, self.rama_remota, self.rama_local
        ));

        self.verificar_remoto()?;
        self.verificar_rama_local()?;
        self.verificar_rama_remota()?;

        self.set_upstream()?;

        self.logger.log(&format!(
            "Se ejecuto set-upstream con exito - remoto: {}, rama remota: {},rama local: {}",
            self.remoto, self.rama_remota, self.rama_local
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

    fn verificar_rama_remota(&self) -> Result<(), String> {
        let rama_remota = format!("{}/{}", self.remoto, self.rama_remota);
        if !utils::ramas::existe_la_rama_remota(&rama_remota) {
            return Err(format!(
                "Rama remota desconocida: {}\n No se puede usar set-upstream\n",
                self.rama_local
            ));
        }

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
#[cfg(test)]

mod tests {
    use std::{path::PathBuf, sync::Arc};

    use crate::{
        tipos_de_dato::{
            comandos::{branch::Branch, init::Init, remote::Remote},
            config::Config,
            logger::Logger,
        },
        utils::io,
    };

    use super::SetUpstream;

    fn limpiar_archivo_gir(logger: Arc<Logger>) {
        elimar_archivo_gir();

        let init = Init {
            path: "./.gir".to_string(),
            logger,
        };
        init.ejecutar().unwrap();
    }

    fn anadir_remoto_default_config(remoto: String, logger: Arc<Logger>) {
        let mut args_remote = vec!["add".to_string(), remoto, "url".to_string()];
        Remote::from(&mut args_remote, logger)
            .unwrap()
            .ejecutar()
            .unwrap();
    }

    //crear la carpeta del
    fn escribir_rama_remota(remoto: String, nombre_rama_remota: String) {
        let dir = format!("./.gir/refs/remotes/{}/{}", remoto, nombre_rama_remota);
        io::escribir_bytes(dir, "contenido").unwrap();
    }

    fn escribir_rama_local(rama: String, logger: Arc<Logger>) {
        let mut branch_args = vec![rama];
        Branch::from(&mut branch_args, logger)
            .unwrap()
            .ejecutar()
            .unwrap();
    }

    fn elimar_archivo_gir() {
        if PathBuf::from("./.gir").exists() {
            io::rm_directorio(".gir").unwrap();
        }
    }

    #[test]
    fn test_01_se_agrega_correctamente_la_configuracion_a_una_rama() {
        let logger = Arc::new(Logger::new("tmp/set_up_stream_01".into()).unwrap());
        let remoto = "origin".to_string();
        let rama_remota = "trabajo".to_string();
        let rama_local = "trabajando".to_string();

        limpiar_archivo_gir(logger.clone());
        anadir_remoto_default_config(remoto.clone(), logger.clone());
        escribir_rama_local(rama_local.clone(), logger.clone());
        escribir_rama_remota(remoto.clone(), rama_remota.clone());

        SetUpstream::new(
            remoto.clone(),
            rama_remota.clone(),
            rama_local.clone(),
            logger,
        )
        .unwrap()
        .ejecutar()
        .unwrap();

        let config = Config::leer_config().unwrap();
        assert!(config.existe_rama(&rama_local));

        let (remoto_obtenido, rama_merge_obtendia) = config
            .obtener_remoto_y_rama_merge_rama(&rama_local)
            .unwrap();
        let rama_merge_esperada = PathBuf::from(format!("refs/heads/{}", rama_remota));

        assert_eq!(remoto_obtenido, remoto);
        assert_eq!(rama_merge_esperada, rama_merge_obtendia);
    }

    #[test]
    fn test_02_se_puede_modificar_el_seteado_de_la_rama() {
        let logger = Arc::new(Logger::new("tmp/set_up_stream_02".into()).unwrap());
        let remoto = "origin".to_string();
        let rama_remota = "trabajo".to_string();
        let rama_local = "trabajando".to_string();
        let rama_remota_2 = "rust".to_string();
        let remoto_2 = "tp".to_string();

        limpiar_archivo_gir(logger.clone());
        anadir_remoto_default_config(remoto.clone(), logger.clone());
        anadir_remoto_default_config(remoto_2.clone(), logger.clone());
        escribir_rama_local(rama_local.clone(), logger.clone());
        escribir_rama_remota(remoto.clone(), rama_remota.clone());
        escribir_rama_remota(remoto_2.clone(), rama_remota_2.clone());

        SetUpstream::new(
            remoto.clone(),
            rama_remota.clone(),
            rama_local.clone(),
            logger.clone(),
        )
        .unwrap()
        .ejecutar()
        .unwrap();

        SetUpstream::new(
            remoto_2.clone(),
            rama_remota_2.clone(),
            rama_local.clone(),
            logger.clone(),
        )
        .unwrap()
        .ejecutar()
        .unwrap();

        let (remoto_obtenido, rama_merge_obtendia) = Config::leer_config()
            .unwrap()
            .obtener_remoto_y_rama_merge_rama(&rama_local)
            .unwrap();
        let rama_merge_esperada = PathBuf::from(format!("refs/heads/{}", rama_remota_2));

        assert_eq!(remoto_obtenido, remoto_2);
        assert_eq!(rama_merge_esperada, rama_merge_obtendia);
    }

    #[test]
    #[should_panic]
    fn test_03_no_se_puede_setear_un_remoto_que_no_existe() {
        let logger = Arc::new(Logger::new("tmp/set_up_stream_03".into()).unwrap());
        let rama_remota = "trabajo".to_string();
        let rama_local = "trabajando".to_string();
        let remoto = "origin".to_string();

        limpiar_archivo_gir(logger.clone());
        escribir_rama_local(rama_local.clone(), logger.clone());
        escribir_rama_remota(remoto.clone(), rama_remota.clone());

        SetUpstream::new(
            remoto.clone(),
            rama_remota.clone(),
            rama_local.clone(),
            logger.clone(),
        )
        .unwrap()
        .ejecutar()
        .unwrap();
    }

    #[test]
    #[should_panic]
    fn test_04_no_se_puede_setear_una_rama_local_que_no_exite() {
        let logger = Arc::new(Logger::new("tmp/set_up_stream_04".into()).unwrap());
        let rama_remota = "trabajo".to_string();
        let rama_local = "trabajando".to_string();
        let remoto = "origin".to_string();

        limpiar_archivo_gir(logger.clone());
        anadir_remoto_default_config(remoto.clone(), logger.clone());
        escribir_rama_remota(remoto.clone(), rama_remota.clone());

        SetUpstream::new(
            remoto.clone(),
            rama_remota.clone(),
            rama_local.clone(),
            logger.clone(),
        )
        .unwrap()
        .ejecutar()
        .unwrap();
    }

    #[test]
    #[should_panic]
    fn test_05_no_se_puede_setear_una_rama_remota_que_no_exite() {
        let logger = Arc::new(Logger::new("tmp/set_up_stream_05".into()).unwrap());
        let rama_remota = "trabajo".to_string();
        let rama_local = "trabajando".to_string();
        let remoto = "origin".to_string();

        limpiar_archivo_gir(logger.clone());
        anadir_remoto_default_config(remoto.clone(), logger.clone());
        escribir_rama_local(rama_local.clone(), logger.clone());

        SetUpstream::new(
            remoto.clone(),
            rama_remota.clone(),
            rama_local.clone(),
            logger.clone(),
        )
        .unwrap()
        .ejecutar()
        .unwrap();
    }
}

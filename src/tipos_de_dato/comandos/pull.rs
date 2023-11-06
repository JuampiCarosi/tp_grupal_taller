use std::{net::TcpStream, path::PathBuf, sync::Arc};

use crate::{
    tipos_de_dato::{comunicacion::Comunicacion, logger::Logger},
    utils::{self, io},
};

use super::{fetch::Fetch, merge::Merge};

const UBICACION_RAMA_MASTER: &str = "./.gir/refs/heads/master";
pub struct Pull {
    rama_actual: String,
    remoto: String,
    logger: Arc<Logger>,
    comunicacion: Arc<Comunicacion<TcpStream>>,
}

impl Pull {
    pub fn from(
        logger: Arc<Logger>,
        comunicacion: Arc<Comunicacion<TcpStream>>,
    ) -> Result<Pull, String> {
        let rama_actual = Self::obtener_rama_actual()?;
        let remoto = "origin".to_string(); //momento, necesita ser el mismo que fetch

        Ok(Pull {
            rama_actual,
            remoto,
            logger,
            comunicacion,
        })
    }
    fn obtener_rama_actual() -> Result<String, String> {
        let dir_rama_actual = Self::obtener_dir_rama_actual()?;
        let rama = utils::path_buf::obtener_nombre(&dir_rama_actual)?;
        Ok(rama)
    }

    fn obtener_dir_rama_actual() -> Result<PathBuf, String> {
        let contenido_head = io::leer_a_string("./.gir/HEAD")?;
        let (_, dir_rama_actual) = contenido_head
            .split_once(' ')
            .ok_or(format!("Fallo al obtener la rama actual\n"))?;
        Ok(PathBuf::from(dir_rama_actual.trim()))
    }

    pub fn ejecutar(&self) -> Result<String, String> {
        self.comunicacion.iniciar_git_upload_pack_con_servidor()?;
        let fetch = Fetch::<TcpStream>::new(self.logger.clone(), self.comunicacion.clone())?;
        //en caso de clone el commit head se tiene que utilizar
        let (
            capacidades_servidor,
            commit_head_remoto,
            commits_cabezas_y_dir_rama_asosiado,
            _commits_y_tags_asosiados,
        ) = fetch.fase_de_descubrimiento()?;

        if !fetch.fase_de_negociacion(capacidades_servidor, &commits_cabezas_y_dir_rama_asosiado)? {
            return Ok(String::from("El cliente esta actualizado"));
        }

        fetch.recivir_packfile_y_guardar_objetos()?;

        fetch.actualizar_ramas_locales_del_remoto(&commits_cabezas_y_dir_rama_asosiado)?;

        self.actualizar_master_de_ser_necesario(commit_head_remoto)?;

        self.mergear_rama()?;

        let mensaje = format!("Pull ejecutado con exito");
        self.logger.log(mensaje.clone());
        Ok(mensaje)
    }

    fn actualizar_master_de_ser_necesario(
        &self,
        commit_head_remoto: Option<String>,
    ) -> Result<bool, String> {
        if !io::esta_vacio(UBICACION_RAMA_MASTER.to_string())? {
            return Ok(false);
        }

        match commit_head_remoto {
            Some(commit) => io::escribir_bytes(UBICACION_RAMA_MASTER, commit)?,
            None => return Ok(false),
        }

        Ok(true)
    }
    fn mergear_rama(&self) -> Result<(), String> {
        let rama_a_mergear = format!("{}/{}", &self.remoto, &self.rama_actual);

        Merge::from(&mut vec![rama_a_mergear], self.logger.clone())?;

        Ok(())
    }
}

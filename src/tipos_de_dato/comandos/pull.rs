use std::{net::TcpStream, path::PathBuf, sync::Arc};

use crate::{
    tipos_de_dato::{
        comandos::write_tree, comunicacion::Comunicacion, logger::Logger, objetos::tree::Tree,
    },
    utils::{
        self,
        io::{self, leer_a_string},
    },
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
            .ok_or("Fallo al obtener la rama actual\n".to_string())?;
        Ok(PathBuf::from(dir_rama_actual.trim()))
    }

    fn obtener_head_remoto(&self) -> Result<String, String> {
        let path_remoto = format!("./.gir/refs/remotes/{}/{}", self.remoto, self.rama_actual);
        leer_a_string(path_remoto)
    }

    pub fn ejecutar(&self) -> Result<String, String> {
        self.comunicacion.iniciar_git_upload_pack_con_servidor()?;
        let fetch = Fetch::<TcpStream>::new(self.logger.clone(), self.comunicacion.clone())?;
        //en caso de clone el commit head se tiene que utilizar
        let (
            capacidades_servidor,
            _commit_head_remoto,
            commits_cabezas_y_dir_rama_asosiado,
            _commits_y_tags_asosiados,
        ) = fetch.fase_de_descubrimiento()?;

        if !fetch.fase_de_negociacion(capacidades_servidor, &commits_cabezas_y_dir_rama_asosiado)? {
            return Ok(String::from("El cliente esta actualizado"));
        }

        fetch.recivir_packfile_y_guardar_objetos()?;

        fetch.actualizar_ramas_locales_del_remoto(&commits_cabezas_y_dir_rama_asosiado)?;
        let commit_head_remoto = self.obtener_head_remoto()?;

        if io::esta_vacio(UBICACION_RAMA_MASTER.to_string())? {
            self.fast_forward_de_cero(commit_head_remoto)?;
        } else {
            self.mergear_rama()?;
        }

        let mensaje = "Pull ejecutado con exito".to_string();
        self.logger.log(mensaje.clone());
        Ok(mensaje)
    }

    fn fast_forward_de_cero(&self, commit_head_remoto: String) -> Result<bool, String> {
        io::escribir_bytes(UBICACION_RAMA_MASTER, &commit_head_remoto)?;
        let hash_tree_padre = write_tree::conseguir_arbol_from_hash_commit(
            &commit_head_remoto,
            String::from(".gir/objects/"),
        );
        let tree_branch_a_mergear =
            Tree::from_hash(hash_tree_padre, PathBuf::from("."), self.logger.clone())?;

        tree_branch_a_mergear.escribir_en_directorio()?;

        Ok(true)
    }
    fn mergear_rama(&self) -> Result<(), String> {
        let rama_a_mergear = format!("{}/{}", &self.remoto, &self.rama_actual);
        Merge::from(&mut vec![rama_a_mergear], self.logger.clone())?.ejecutar()?;

        Ok(())
    }
}

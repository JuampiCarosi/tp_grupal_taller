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
        let rama_actual = utils::ramas::obtener_rama_actual()?;
        let remoto = "origin".to_string(); //momento, necesita ser el mismo que fetch
        Ok(Pull {
            rama_actual,
            remoto,
            logger,
            comunicacion,
        })
    }

    pub fn ejecutar(&self) -> Result<String, String> {
        Fetch::<TcpStream>::new(
            vec![self.remoto.clone()],
            self.logger.clone(),
            self.comunicacion.clone(),
        )?
        .ejecutar()?;

        let commit_head_remoto = self.obtener_head_remoto()?;

        if io::esta_vacio(UBICACION_RAMA_MASTER.to_string()) {
            self.fast_forward_de_cero(commit_head_remoto)?;
        } else {
            self.mergear_rama()?;
        }

        let mensaje = format!("Pull ejecutado con exito");
        self.logger.log(mensaje.clone());
        Ok(mensaje)
    }

    fn obtener_head_remoto(&self) -> Result<String, String> {
        let path_remoto = format!("/.gir/{}_HEAD", self.remoto.to_uppercase());
        leer_a_string(path_remoto)
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

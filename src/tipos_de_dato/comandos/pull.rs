use std::{net::TcpStream, path::PathBuf, sync::Arc};

use crate::{
    tipos_de_dato::{comandos::write_tree, config::Config, logger::Logger, objetos::tree::Tree},
    utils::{
        self,
        io::{self, leer_a_string},
    },
};

use super::{fetch::Fetch, merge::Merge};

const UBICACION_RAMA_MASTER: &str = "./.gir/refs/heads/master";
const GIR_PULL: &str = "gir pull <remoto> <rama>";
pub struct Pull {
    rama_merge: String,
    remoto: String,
    logger: Arc<Logger>,
    set_upstream: bool,
}

impl Pull {
    pub fn from(args: Vec<String>, logger: Arc<Logger>) -> Result<Pull, String> {
        Self::verificar_argumentos(&args)?;
        let set_upstream = false;
        // if Self::hay_flags(&args){
        //     if args[0] == "-u".to_string() {}
        // }

        let remoto = Self::obtener_remoto(args)?; //momento, necesita ser el mismo que fetch

        let rama_merge = utils::ramas::obtener_rama_actual()?;

        Ok(Pull {
            rama_merge,
            remoto,
            logger,
            set_upstream,
        })
    }

    fn hay_flags(args: &Vec<String>) -> bool {
        args.len() == 2
    }

    fn parsear_flags(args:){}

    fn verificar_argumentos(args: &Vec<String>) -> Result<(), String> {
        if args.len() > 3 {
            return Err(format!(
                "Parametros desconocidos {}\n {}",
                args.join(" "),
                GIR_PULL
            ));
        };
        Ok(())
    }
    ///obtiene el remoto para el comando, si argumentos lo contiene y es valido lo saca de argumentos. Si no hay argumetos lo saca 
    /// del remoto asosiado a la rama actual. Si no esta configura la rama actual para ningun remoto devuleve error. 
    fn obtener_remoto(args: Vec<String>) -> Result<String, String> {
        let remoto = if args.len() == 1 {
            Self::verificar_remoto(&args[0])?
        } else {
            Self::obtener_remoto_rama_actual()?
        };
        Ok(remoto)
    }

    ///verifica si el remoto envio por el usario existe
    fn verificar_remoto(remoto: &String) -> Result<String, String> {
        if let false = Config::leer_config()?.existe_remote(remoto) {
            return  Err(format!("Remoto desconocido{}\nSi quiere añadir un nuevo remoto:\n\ngir remote add [<nombre-remote>] [<url-remote>]\n\n", remoto));
        };

        Ok(remoto.clone())
    }

    ///obtiene el remo asosiado a la rama remota actual. Falla si no existe
    fn obtener_remoto_rama_actual() -> Result<String, String> {
        Config::leer_config()?
            .obtener_remoto_rama_actual()
            .ok_or(format!(
                "La rama actual no se encuentra asosiado a ningun remoto\nUtilice:\n\ngir remote add [<nombre-remote>] [<url-remote>]\n\nDespues:\n\n{}\n\n", GIR_PULL
            ))
    }

    pub fn ejecutar(&self) -> Result<String, String> {
        Fetch::<TcpStream>::new(vec![self.remoto.clone()], self.logger.clone())?.ejecutar()?;

        let commit_head_remoto = self.obtener_head_remoto()?;

        if io::esta_vacio(UBICACION_RAMA_MASTER.to_string()) {
            self.fast_forward_de_cero(commit_head_remoto)?;
        } else {
            self.mergear_rama()?;
        }

        let mensaje = "Pull ejecutado con exito".to_string();
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
        let rama_a_mergear = format!("{}/{}", &self.remoto, &self.rama_merge);
        Merge::from(&mut vec![rama_a_mergear], self.logger.clone())?.ejecutar()?;

        Ok(())
    }
}

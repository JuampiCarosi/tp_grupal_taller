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
const FLAG_SET_UPSTREAM: &str = "--set-upstream";
const FLAG_U: &str = "-u";

pub struct Pull {
    rama_merge: String,
    remoto: String,
    logger: Arc<Logger>,
    set_upstream: bool,
}

impl Pull {
    pub fn from(mut args: Vec<String>, logger: Arc<Logger>) -> Result<Pull, String> {
        Self::verificar_argumentos(&args)?;

        let set_upstream = false;

        // if Self::hay_flags(&args){
        //     if args[0] == "-u".to_string() {}
        // }
        if args.len() == 2 {}

        let remoto = Self::obtener_remoto(&mut args)?; //momento, necesita ser el mismo que fetch
        let rama_merge = utils::ramas::obtener_rama_actual()?;

        Ok(Pull {
            rama_merge,
            remoto,
            logger,
            set_upstream,
        })
    }

    fn hay_flags(args: &Vec<String>) -> bool {
        args.len() == 3
    }

    fn parsear_flags(mut args: Vec<String>, set_upstream: &mut bool) -> Result<(), String> {
        let flag = args.remove(0);

        if flag == FLAG_U || flag == FLAG_SET_UPSTREAM {
            *set_upstream = true;
            Ok(())
        } else {
            Err(format!(
                "Parametros desconocidos {}\n {}",
                args.join(" "),
                GIR_PULL
            ))
        }
    }

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

    fn obtener_remoto(args: &mut Vec<String>) -> Result<String, String> {
        let mut remoto = String::new();
        let mut rama_merge = String::new();

        if args.len() == 2 {
            remoto = Self::verificar_remoto(&args[0])?;
            rama_merge = args.remove(1);
        } else {
            Self::obtener_remoto_y_rama_merge_de_rama_actual()?;
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
    fn obtener_remoto_y_rama_merge_de_rama_actual() -> Result<(String, PathBuf), String> {
        Config::leer_config()?
            .obtener_remoto_y_rama_merge_rama_actual()
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

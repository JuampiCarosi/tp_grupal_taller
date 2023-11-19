use std::{net::TcpStream, path::PathBuf, sync::Arc};

use crate::{
    tipos_de_dato::{comandos::write_tree, config::Config, logger::Logger, objetos::tree::Tree},
    utils::{
        io::{self, leer_a_string},
        path_buf::obtener_nombre,
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

        let mut set_upstream = false;

        if Self::hay_flags(&args) {
            Self::parsear_flags(&mut args, &mut set_upstream)?;
        }

        let (remoto, rama_merge) = Self::parsear_argumentos(&mut args, set_upstream)?;

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
    ///Setea pull segun los flags recibidos
    fn parsear_flags(args: &mut Vec<String>, set_upstream: &mut bool) -> Result<(), String> {
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

    ///Obtiene acorde a los argumentos recibidos, el remoto y la rama merge. En caso de no estar,
    /// busca si esta seteada la rama actual. Si esto no es asi, hay un error
    fn parsear_argumentos(
        args: &mut Vec<String>,
        set_upstream: bool,
    ) -> Result<(String, String), String> {
        let remoto;
        let rama_merge;

        if args.len() == 2 {
            remoto = Self::verificar_remoto(&args[0])?;
            rama_merge = args.remove(1);
        } else if args.len() == 0 && !set_upstream {
            //si no hay argumentos ni flags, quiere decir que deberia
            //estar configurada la rama
            (remoto, rama_merge) = Self::obtener_remoto_y_rama_merge_de_rama_actual()?;
        } else {
            return Err(format!(
                "Parametros faltantes {}\n {}",
                args.join(" "),
                GIR_PULL
            ));
        }

        Ok((remoto, rama_merge))
    }

    ///verifica si el remoto envio por el usario existe
    fn verificar_remoto(remoto: &String) -> Result<String, String> {
        if let false = Config::leer_config()?.existe_remote(remoto) {
            return  Err(format!("Remoto desconocido{}\nSi quiere añadir un nuevo remoto:\n\ngir remote add [<nombre-remote>] [<url-remote>]\n\n", remoto));
        };

        Ok(remoto.clone())
    }

    ///obtiene el remoto  y la rama merge asosiado a la rama remota actual. Falla si no existe
    fn obtener_remoto_y_rama_merge_de_rama_actual() -> Result<(String, String), String> {
        let (remoto, rama_merge) = Config::leer_config()?
            .obtener_remoto_y_rama_merge_rama_actual()
            .ok_or(format!(
                "La rama actual no se encuentra asosiado a ningun remoto\nUtilice:\n\ngir remote add [<nombre-remote>] [<url-remote>]\n\nDespues:\n\n{}\n\n", GIR_PULL
            ))?;
        //CORREGIR MENSAJE DE ERROR DEBERIA SER QUE USE SET BRANCH

        Ok((remoto, obtener_nombre(&rama_merge)?))
    }

    pub fn ejecutar(&self) -> Result<String, String> {
        if self.set_upstream {
            return Ok("Hacer acordar a Siro, que implemente esto :)".to_string());
        }

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

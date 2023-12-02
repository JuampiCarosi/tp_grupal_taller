use super::set_upstream::SetUpstream;
use crate::tipos_de_dato::comando::Ejecutar;
use crate::tipos_de_dato::comandos::write_tree;
use crate::tipos_de_dato::comunicacion::Comunicacion;
use crate::tipos_de_dato::config::Config;
use crate::tipos_de_dato::logger::Logger;
use crate::tipos_de_dato::objetos::commit::CommitObj;
use crate::tipos_de_dato::objetos::tree::Tree;
use crate::tipos_de_dato::packfile::Packfile;
use crate::utils;
use crate::utils::io;
use crate::utils::path_buf::obtener_nombre;

use std::collections::HashSet;
use std::net::TcpStream;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;

const FLAG_SET_UPSTREAM: &str = "--set-upstream";
const FLAG_U: &str = "-u";
const GIR_PUSH: &str = "gir push <remoto> <rama>";

// idea: Key -> (String, String) , primera entrada la ref que tiene el cliente, segunda la que tiene el sv.
pub struct Push {
    comunicacion: Comunicacion<TcpStream>,
    rama_merge: String,
    remoto: String,
    set_upstream: bool,
    logger: Arc<Logger>,
}

impl Push {
    // lo que se hace aca es obtener las referencias del cliente, y setear las del servidor en 0 (40 veces 0)
    // asi queda la estructura del hashmap como: Referencia: (ref_cliente, ref_servidor)
    // donde referencia puede ser por ejemplo: refs/heads/master

    pub fn new(args: &mut Vec<String>, logger: Arc<Logger>) -> Result<Self, String> {
        Self::verificar_argumentos(args)?;

        let mut set_upstream = false;

        if Self::hay_flags(&args) {
            Self::parsear_flags(args, &mut set_upstream)?;
        }

        let (remoto, rama_merge) = Self::parsear_argumentos(args, set_upstream)?;

        let url: String = Self::obtener_url(&remoto)?;
        let comunicacion = Comunicacion::<TcpStream>::new_desde_url(&url, logger.clone())?;

        Ok(Push {
            comunicacion,
            rama_merge,
            remoto,
            set_upstream,
            logger,
        })
    }

    fn verificar_argumentos(args: &Vec<String>) -> Result<(), String> {
        if args.len() > 3 {
            return Err(format!(
                "Parametros desconocidos {}\n {}",
                args.join(" "),
                GIR_PUSH
            ));
        };
        Ok(())
    }

    fn hay_flags(args: &Vec<String>) -> bool {
        args.len() == 3
    }

    ///obtiene el remoto  y la rama merge asosiado a la rama remota actual. Falla si no existe
    fn obtener_remoto_y_rama_merge_de_rama_actual() -> Result<(String, String), String> {
        let (remoto, rama_merge) = Config::leer_config()?
            .obtener_remoto_y_rama_merge_rama_actual()
            .ok_or("La rama actual no se encuentra asosiado a ningun remoto\nUtilice: gir push --set-upstream/-u nombre-remoto nombre-rama-local".to_string())?;
        //CORREGIR MENSAJE DE ERROR DEBERIA SER QUE USE SET BRANCH

        Ok((remoto, obtener_nombre(&rama_merge)?))
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
        } else if args.is_empty() && !set_upstream {
            //si no hay argumentos ni flags, quiere decir que deberia
            //estar configurada la rama
            (remoto, rama_merge) = Self::obtener_remoto_y_rama_merge_de_rama_actual()?;
        } else {
            return Err(format!(
                "Parametros faltantes {}\n {}",
                args.join(" "),
                GIR_PUSH
            ));
        }

        Ok((remoto, rama_merge))
    }

    fn parsear_flags(args: &mut Vec<String>, set_upstream: &mut bool) -> Result<(), String> {
        let flag = args.remove(0);

        if flag == FLAG_U || flag == FLAG_SET_UPSTREAM {
            *set_upstream = true;
            Ok(())
        } else {
            Err(format!(
                "Parametros desconocidos {}\n {}",
                args.join(" "),
                GIR_PUSH
            ))
        }
    }
    //Le pide al config el url asosiado a la rama
    fn obtener_url(remoto: &str) -> Result<String, String> {
        Config::leer_config()?.obtenet_url_asosiado_remoto(remoto)
    }

    fn verificar_remoto(remoto: &str) -> Result<String, String> {
        if let false = Config::leer_config()?.existe_remote(remoto) {
            return  Err(format!("Remoto desconocido{}\nSi quiere añadir un nuevo remoto:\n\ngir remote add [<nombre-remote>] [<url-remote>]\n\n", remoto));
        };

        Ok(remoto.to_string())
    }

    //obtiene todo los objetos de una referencia hasta el viejo commit. Si no esta el viejo commit entonces termina la comunicacion
    //y envia un pack file vacio
    fn obtener_objetos_a_enviar(
        &self,
        referencia: &str,
        viejo_commit: &str,
    ) -> Result<HashSet<String>, String> {
        let objetos_a_enviar = Self::obtener_commits_y_objetos_asociados(
            referencia,
            viejo_commit,
            self.logger.clone(),
        );

        match objetos_a_enviar {
            Ok(objetos_a_enviar) => Ok(objetos_a_enviar),
            Err(msj_err) => {
                //error
                self.comunicacion.enviar_flush_pkt()?;
                // el server pide que se le mande un packfile vacio
                self.comunicacion
                    .enviar_pack_file(Packfile::obtener_pack_con_archivos(
                        vec![],
                        "./.gir/objects/",
                    )?)?;
                Err(msj_err)
            }
        }
    }

    ///Obtiene la referecia que hay que actulizar del servidor y todos sus componentes(viejo commit, nuevo commit y ref rama).
    /// Para obtener el viejo commit compara el nombre de la ref con los ref ramas recibidos del servidor y lo busca.
    /// Si no existe viejo commit se completa con ceros (00..00).
    ///
    /// ## Argumentos
    /// -   commits_cabezas_y_ref_rama_asosiado: vector de tuplas de commit y su rama ref asosiado
    ///
    /// ## Resultado
    /// - Tupla con:
    ///     - el commit viejo
    ///     - el commit nuevo
    ///     - la ref de la rama
    fn obtener_referencia_acualizar(
        &self,
        commits_cabezas_y_ref_rama_asosiado: &Vec<(String, PathBuf)>,
    ) -> Result<(String, String, String), String> {
        let mut commit_viejo = "0".repeat(40);
        let commit_nuevo = io::leer_a_string(utils::ramas::obtener_gir_dir_rama_actual()?)?;
        let ref_rama_merge = format!("refs/heads/{}", self.rama_merge);

        for (commit, referencia) in commits_cabezas_y_ref_rama_asosiado {
            if *referencia.to_string_lossy() == ref_rama_merge {
                commit_viejo = commit.to_string();
            }
        }

        Ok((commit_viejo, commit_nuevo, ref_rama_merge))
    }

    fn enviar_actualizaciones_y_objetos(
        &mut self,
        referencia_actualizar: (String, String, String),
        objetos_a_enviar: HashSet<String>,
    ) -> Result<(), String> {
        self.comunicacion
            .enviar(&utils::io::obtener_linea_con_largo_hex(&format!(
                "{} {} {}",
                referencia_actualizar.0, referencia_actualizar.1, referencia_actualizar.2
            )))?;

        self.comunicacion.enviar_flush_pkt()?;

        self.comunicacion
            .enviar_pack_file(Packfile::obtener_pack_con_archivos(
                objetos_a_enviar.into_iter().collect(),
                "./.gir/objects/",
            )?)?;
        Ok(())
    }

    ///Se encarga de la fase de descubrimiento con el servidor, en la cual se recibe del servidor
    /// una lista de referencias.
    /// La primera linea contiene la version del server
    /// La segunda linea recibida tiene el siguiente : 'hash_del_commit_head HEAD'\0'lista de capacida'
    /// Las siguients lineas: 'hash_del_commit_cabeza_de_rama_en_el_servidor'
    ///                        'direccion de la carpeta de la rama en el servidor'
    ///
    /// # Resultado
    /// - vector con las capacidades del servidor
    /// - hash del commit cabeza de rama
    /// - vector de tuplas con los hash del commit cabeza de rama y la ref de la
    ///     de la rama en el servidor(ojo!! la direccion para el servidor no para el local)
    /// - vector de tuplas con el hash del commit y el tag asosiado
    fn fase_de_descubrimiento(
        &self,
    ) -> Result<
        (
            Vec<String>,
            Option<String>,
            Vec<(String, PathBuf)>,
            Vec<(String, PathBuf)>,
        ),
        String,
    > {
        utils::fase_descubrimiento::fase_de_descubrimiento(&self.comunicacion)
    }

    // ------ funciones auxiliares ------

    // funcion para obtener los commits que faltan para llegar al commit limite y los objetos asociados a cada commit
    // en caso de que sea una referencia nula, se enviara todo. En caso de que el commit limite no sea una referencia nula
    // y no se encuentre al final de la cadena de commits, se enviara un error, ya que el servidor tiene cambios que el cliente no tiene
    fn obtener_commits_y_objetos_asociados(
        referencia: &str,
        commit_limite: &str,
        _logger: Arc<Logger>,
    ) -> Result<HashSet<String>, String> {
        let logger = Arc::new(Logger::new(PathBuf::from("./tmp/aa"))?);
        let ruta = format!(".gir/{}", referencia);
        let ultimo_commit = io::leer_a_string(Path::new(&ruta))?;
        if ultimo_commit.is_empty() {
            return Ok(HashSet::new());
        }

        // let mut objetos_a_agregar: HashMap<String, CommitObj> = HashMap::new();
        let mut objetos_a_agregar: HashSet<String> = HashSet::new();
        let mut commits_a_revisar: Vec<CommitObj> = Vec::new();

        let ultimo_commit = CommitObj::from_hash(ultimo_commit, logger.clone());

        match ultimo_commit {
            Ok(ultimo_commit) => {
                commits_a_revisar.push(ultimo_commit);
            }
            Err(_) => {
                return Err(
                    "El servidor tiene cambios, por favor, actualice su repositorio".to_string(),
                );
            }
        }

        while let Some(commit) = commits_a_revisar.pop() {
            if objetos_a_agregar.contains(&commit.hash) {
                continue;
            }
            if commit.hash == commit_limite {
                objetos_a_agregar.insert(commit.hash.clone());
                break;
            }
            objetos_a_agregar.insert(commit.hash.clone());
            let hash_tree =
                write_tree::conseguir_arbol_from_hash_commit(&commit.hash, "./.gir/objects/")?;
            let tree = Tree::from_hash(&hash_tree, PathBuf::from("."), logger.clone())?;
            objetos_a_agregar.insert(hash_tree);
            objetos_a_agregar.extend(
                tree.obtener_objetos()
                    .iter()
                    .map(|objeto| objeto.obtener_hash()),
            );

            for padre in commit.padres {
                let commit_padre = CommitObj::from_hash(padre, logger.clone())?;
                commits_a_revisar.push(commit_padre);
            }
        }
        if ("0".repeat(40) != *commit_limite) && !objetos_a_agregar.contains(commit_limite) {
            return Err(
                "El servidor tiene cambios, por favor, actualice su repositorio".to_string(),
            );
        } else if ("0".repeat(40) != *commit_limite) && objetos_a_agregar.contains(commit_limite) {
            objetos_a_agregar.remove(commit_limite);
        }
        Ok(objetos_a_agregar)
    }
}

impl Ejecutar for Push {
    fn ejecutar(&mut self) -> Result<String, String> {
        if self.set_upstream {
            SetUpstream::new(
                self.remoto.clone(),
                self.rama_merge.clone(),
                utils::ramas::obtener_rama_actual()?,
                self.logger.clone(),
            )?
            .ejecutar()?;
        }

        self.comunicacion.iniciar_git_recive_pack_con_servidor()?;
        let (
            _capacidades_servidor,
            _commit_head_remoto,
            commits_cabezas_y_ref_rama_asosiado,
            _commits_y_tags_asosiados,
        ) = self.fase_de_descubrimiento()?;
        self.logger.log("Fase de descubrimiento ejecuta con exito");

        println!("{:?}", commits_cabezas_y_ref_rama_asosiado);
        let referencia_acualizar =
            self.obtener_referencia_acualizar(&commits_cabezas_y_ref_rama_asosiado)?;
        let objetos_a_enviar =
            self.obtener_objetos_a_enviar(&referencia_acualizar.2, &referencia_acualizar.0)?;

        self.enviar_actualizaciones_y_objetos(referencia_acualizar, objetos_a_enviar)?;

        let mensaje = "Push ejecutado con exito".to_string();
        self.logger.log(&mensaje);
        Ok(mensaje)
    }
}

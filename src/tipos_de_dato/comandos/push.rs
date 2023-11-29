use super::set_upstream::SetUpstream;
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
use std::collections::HashMap;
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
        Self::verificar_argumentos(&args)?;

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
            .ok_or(format!(
                "La rama actual no se encuentra asosiado a ningun remoto\nUtilice: gir push --set-upstream/-u nombre-remoto nombre-rama-local"))?;
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
        } else if args.len() == 0 && !set_upstream {
            println!("Entre aca");
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
    fn obtener_url(remoto: &String) -> Result<String, String> {
        Config::leer_config()?.obtenet_url_asosiado_remoto(&remoto)
    }

    fn verificar_remoto(remoto: &String) -> Result<String, String> {
        if let false = Config::leer_config()?.existe_remote(remoto) {
            return  Err(format!("Remoto desconocido{}\nSi quiere añadir un nuevo remoto:\n\ngir remote add [<nombre-remote>] [<url-remote>]\n\n", remoto));
        };

        Ok(remoto.clone())
    }

    // recibe las referencia junto a la version y las capacidades del servidor.
    fn obtener_referencias_y_capacidades(&mut self) -> Result<(Vec<String>, String), String> {
        let mut refs_recibidas = self.comunicacion.obtener_lineas()?;

        let _version = refs_recibidas.remove(0);
        let first_ref = refs_recibidas.remove(0);

        let referencia_y_capacidades = first_ref.split('\0').collect::<Vec<&str>>();
        let referencia = referencia_y_capacidades[0].to_string();
        let capacidades = referencia_y_capacidades[1].to_string();
        if !referencia.contains(&"0".repeat(40)) {
            refs_recibidas.push(referencia_y_capacidades[0].to_string());
        }
        Ok((refs_recibidas, capacidades))
    }

    // funcion para guaradar en el hashmap las diferencias entre las refs del cliente y las del server
    fn guardar_diferencias(
        &mut self,
        refs_recibidas: Vec<String>,
        hash_refs: &mut HashMap<String, (String, String)>,
    ) -> Result<(), String> {
        for referencia in &refs_recibidas {
            let obj_id = referencia.split(' ').collect::<Vec<&str>>()[0];
            let referencia = referencia.split(' ').collect::<Vec<&str>>()[1].trim_end_matches('\n');
            match hash_refs.get_mut(referencia) {
                Some(hash) => {
                    hash.1 = obj_id.to_string();
                }
                None => {}
            }
        }
        Ok(())
    }

    // funcion que devuelve los objetos que hay que enviar al server y las actualizaciones que hay que hacer
    fn obtener_objetos_a_enviar(
        &self,
        hash_refs: &HashMap<String, (String, String)>,
    ) -> Result<(HashSet<String>, Vec<String>), String> {
        let mut actualizaciones = Vec::new();
        let mut objetos_a_enviar = HashSet::new();

        for (key, value) in hash_refs {
            if value.1 != value.0 {
                actualizaciones.push(io::obtener_linea_con_largo_hex(&format!(
                    "{} {} {}",
                    &value.1, &value.0, &key
                )));
                let nuevos_objetos = obtener_commits_y_objetos_asociados(key, &value.1);
                match nuevos_objetos {
                    Ok(nuevos_objetos) => {
                        objetos_a_enviar.extend(nuevos_objetos);
                    }
                    Err(err) => {
                        //error
                        self.comunicacion.responder(vec![]).unwrap();
                        // el server pide que se le mande un packfile vacio
                        self.comunicacion
                            .responder_con_bytes(
                                Packfile::new()
                                    .obtener_pack_con_archivos(vec![], "./.gir/objects/"),
                            )
                            .unwrap();
                        return Err(err);
                    }
                }
            }
        }
        Ok((objetos_a_enviar, actualizaciones))
    }

    fn enviar_actualizaciones_y_objetos(
        &mut self,
        actualizaciones: Vec<String>,
        objetos_a_enviar: HashSet<String>,
    ) -> Result<String, String> {
        if !actualizaciones.is_empty() {
            self.comunicacion.responder(actualizaciones).unwrap();
            self.comunicacion
                .responder_con_bytes(Packfile::new().obtener_pack_con_archivos(
                    objetos_a_enviar.into_iter().collect(),
                    "./.gir/objects/",
                ))
                .unwrap();
            Ok(String::from("Push ejecutado con exito"))
        } else {
            //error
            Err("No hay actualizaciones".to_string())
        }
    }

    fn obtener_nombre_referencia_y_commit_actulizar(&self) -> Result<(String, String), String> {
        let commit_head_rama_actual = io::leer_a_string(utils::ramas::obtener_dir_rama_actual()?)?;
        let ref_rama_merge = format!("ref/heads/{}", self.rama_merge);

        Ok((ref_rama_merge, commit_head_rama_actual))
    }

    pub fn ejecutar(&mut self) -> Result<String, String> {
        if self.set_upstream {
            SetUpstream::new(
                self.remoto.clone(),
                self.rama_merge.clone(),
                utils::ramas::obtener_rama_actual()?,
                self.logger.clone(),
            )?
            .ejecutar()?;
        }

        let referencia_actualizar = self.obtener_nombre_referencia_y_commit_actulizar()?;

        // let refs = obtener_refs_de(PathBuf::from("./.gir/refs/"), String::from("./.gir/"));
        // let mut hash_refs: HashMap<String, (String, String)> = HashMap::new();
        // for referencia in refs {
        //     hash_refs.insert(
        //         referencia.split(' ').collect::<Vec<&str>>()[1].to_string(),
        //         (
        //             referencia.split(' ').collect::<Vec<&str>>()[0].to_string(),
        //             "0".repeat(40),
        //         ),
        //     );
        // }

        self.comunicacion.iniciar_git_recive_pack_con_servidor()?;
        let (refs_recibidas, _capacidades) = self.obtener_referencias_y_capacidades()?;

        self.guardar_diferencias(refs_recibidas, &mut hash_refs)?;

        let (objetos_a_enviar, actualizaciones) = self.obtener_objetos_a_enviar(&hash_refs)?;
        self.enviar_actualizaciones_y_objetos(actualizaciones, objetos_a_enviar)
    }
}

// ------ funciones auxiliares ------

// funcion para obtener los commits que faltan para llegar al commit limite y los objetos asociados a cada commit
// en caso de que sea una referencia nula, se enviara todo. En caso de que el commit limite no sea una referencia nula
// y no se encuentre al final de la cadena de commits, se enviara un error, ya que el servidor tiene cambios que el cliente no tiene
fn obtener_commits_y_objetos_asociados(
    referencia: &String,
    commit_limite: &String,
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

    let ultimo_commit = CommitObj::from_hash(ultimo_commit);

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
        if commit.hash == commit_limite.clone() {
            objetos_a_agregar.insert(commit.hash.clone());
            break;
        }
        objetos_a_agregar.insert(commit.hash.clone());
        let hash_tree = write_tree::conseguir_arbol_from_hash_commit(
            &commit.hash,
            "./.gir/objects/".to_string(),
        );
        let tree = Tree::from_hash(hash_tree.clone(), PathBuf::from("."), logger.clone())?;
        objetos_a_agregar.insert(hash_tree.clone());
        objetos_a_agregar.extend(
            tree.obtener_objetos()
                .iter()
                .map(|objeto| objeto.obtener_hash()),
        );

        for padre in commit.padres {
            let commit_padre = CommitObj::from_hash(padre)?;
            commits_a_revisar.push(commit_padre);
        }
    }
    if (commit_limite != &"0".repeat(40)) && !objetos_a_agregar.contains(&commit_limite.clone()) {
        return Err("El servidor tiene cambios, por favor, actualice su repositorio".to_string());
    } else if (commit_limite != &"0".repeat(40))
        && objetos_a_agregar.contains(&commit_limite.clone())
    {
        objetos_a_agregar.remove(commit_limite);
    }
    Ok(objetos_a_agregar)
}

fn obtener_refs_de(dir: PathBuf, prefijo: String) -> Vec<String> {
    let mut refs: Vec<String> = Vec::new();
    refs.append(&mut io::obtener_refs(dir.join("heads/"), prefijo.clone()).unwrap());
    refs.append(&mut io::obtener_refs(dir.join("tags/"), prefijo).unwrap());
    refs
}

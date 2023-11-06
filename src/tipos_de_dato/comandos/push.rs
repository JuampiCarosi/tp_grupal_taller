use crate::tipos_de_dato::comandos::write_tree;
use crate::tipos_de_dato::comunicacion::Comunicacion;
use crate::tipos_de_dato::logger::Logger;
use crate::tipos_de_dato::objetos::commit::CommitObj;
use crate::tipos_de_dato::objetos::tree::Tree;
use crate::tipos_de_dato::packfile::Packfile;
use crate::utils::io;
use std::collections::HashMap;
use std::collections::HashSet;
use std::net::TcpStream;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
// idea: Key -> (String, String) , primera entrada la ref que tiene el cliente, segunda la que tiene el sv.
pub struct Push {
    hash_refs: HashMap<String, (String, String)>,
    comunicacion: Arc<Comunicacion<TcpStream>>,
}

impl Push {
    pub fn new(comunicacion: Arc<Comunicacion<TcpStream>>) -> Self {
        let mut hash_refs: HashMap<String, (String, String)> = HashMap::new();
        let refs = obtener_refs_de(PathBuf::from("./.gir/refs/"), String::from("./.gir/"));
        for referencia in refs {
            hash_refs.insert(
                referencia.split(' ').collect::<Vec<&str>>()[1].to_string(),
                (
                    referencia.split(' ').collect::<Vec<&str>>()[0].to_string(),
                    "0".repeat(40),
                ),
            );
        }
        Push {
            hash_refs,
            comunicacion,
        }
    }

    pub fn ejecutar(&mut self) -> Result<String, String> {
        println!("Se ejecuto el comando push");
        let request_data = "git-receive-pack /gir/\0host=example.com\0\0version=1\0"; //en donde dice /.git/ va la dir del repo

        let request_data_con_largo_hex = io::obtener_linea_con_largo_hex(request_data);

        self.comunicacion.enviar(&request_data_con_largo_hex)?;
        println!("Le mande las cosas");
        let mut refs_recibidas = self.comunicacion.obtener_lineas().unwrap();
        println!("refs recibidas: {:?}", refs_recibidas);
        let mut actualizaciones = Vec::new();
        let mut objetos_a_enviar = HashSet::new();

        let _version = refs_recibidas.remove(0);
        let first_ref = refs_recibidas.remove(0);
        // caso en el que el server no tiene refs
        if first_ref.contains(&"0".repeat(40)) {
            let referencia_y_capacidades = first_ref.split('\0').collect::<Vec<&str>>();
            let capacidades = referencia_y_capacidades[0].to_string();
            // refs_recibidas.push(referencia_y_capacidades[0].to_string());
        }
        println!("refs recibidas: {:?}", refs_recibidas);
        for referencia in &refs_recibidas {
            let obj_id = referencia.split(' ').collect::<Vec<&str>>()[0];
            let referencia = referencia.split(' ').collect::<Vec<&str>>()[1];
            println!("referencia: {}", referencia);
            match self.hash_refs.get_mut(referencia) {
                Some(hash) => {
                    hash.1 = obj_id.to_string();
                }
                None => {
                    // el server tiene un head que el cliente no tiene, abortar push (no borramos brancahes por lo tanto el sv esta por delante)
                    // return Err("El servidor tiene un head que el cliente no tiene".to_string());
                }
            }
        }

        println!("hash_refs: {:?}", self.hash_refs);

        for (key, value) in &self.hash_refs {
            actualizaciones.push(io::obtener_linea_con_largo_hex(&format!(
                "{} {} {}",
                &value.1, &value.0, &key
            ))); // viejo (el del sv), nuevo (cliente), ref
                 // checkear que no existan los objetos antes de appendear
            if value.1 == "0".repeat(40) {
                objetos_a_enviar
                    .extend(obtener_commits_y_objetos_asociados(&key, &value.0).unwrap());
                continue;
            }
            if value.1 != value.0 {
                objetos_a_enviar
                    .extend(obtener_commits_y_objetos_asociados(&key, &value.1).unwrap());
            }
        }
        println!("objetos a enviar: {:?}", objetos_a_enviar);

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
            return Err("No hay actualizaciones".to_string());
        }

        // println!("Refs recibidas: {:?}", refs_recibidas);
    }
}

fn obtener_commits_y_objetos_asociados(
    referencia: &String,
    commit_limite: &String,
) -> Result<HashSet<String>, String> {
    println!(
        "Entro para la referencia {} y el commit limite: {}",
        referencia, commit_limite
    );
    let logger = Arc::new(Logger::new(PathBuf::from("./tmp/aa"))?);
    let ruta = format!(".gir/{}", referencia);
    println!("ruta: {}", ruta);
    let ultimo_commit = io::leer_a_string(Path::new(&ruta))?;
    if ultimo_commit.is_empty() {
        return Ok(HashSet::new());
    }
    println!("ultimo commit: {}", ultimo_commit);

    // let mut objetos_a_agregar: HashMap<String, CommitObj> = HashMap::new();
    let mut objetos_a_agregar: HashSet<String> = HashSet::new();
    let mut commits_a_revisar: Vec<CommitObj> = Vec::new();
    commits_a_revisar.push(CommitObj::from_hash(ultimo_commit)?);

    while let Some(commit) = commits_a_revisar.pop() {
        if objetos_a_agregar.contains(&commit.hash) || commit.hash == *commit_limite {
            objetos_a_agregar.insert(commit.hash.clone());
            let hash_tree = write_tree::conseguir_arbol_from_hash_commit(
                &commit.hash,
                "./.gir/objects/".to_string(),
            );
            let tree = Tree::from_hash(
                hash_tree.clone(),
                PathBuf::from("./.gir/objects/"),
                logger.clone(),
            )?;
            objetos_a_agregar.insert(hash_tree);
            objetos_a_agregar.extend(
                tree.obtener_objetos()
                    .iter()
                    .map(|objeto| objeto.obtener_hash()),
            );
            break;
        }
        objetos_a_agregar.insert(commit.hash.clone());
        let hash_tree = write_tree::conseguir_arbol_from_hash_commit(
            &commit.hash,
            "./.gir/objects/".to_string(),
        );
        let tree = Tree::from_hash(
            hash_tree.clone(),
            PathBuf::from("./.gir/objects/"),
            logger.clone(),
        )?;
        objetos_a_agregar.insert(hash_tree);
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

    // let mut commits_vec = Vec::from_iter(commits.values().cloned());
    // commits_vec.sort_by_key(|commit| commit.date.tiempo.clone());

    Ok(objetos_a_agregar)
}

fn obtener_refs_de(dir: PathBuf, prefijo: String) -> Vec<String> {
    let mut refs: Vec<String> = Vec::new();
    refs.append(&mut io::obtener_refs(dir.join("heads/"), prefijo.clone()).unwrap());
    refs.append(&mut io::obtener_refs(dir.join("tags/"), prefijo).unwrap());
    // refs[0] = self.agregar_capacidades(refs[0].clone ());
    refs
}

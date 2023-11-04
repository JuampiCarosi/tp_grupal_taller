use crate::packfile::Packfile;
use crate::{comunicacion::Comunicacion, io};
use std::io::Write;
use std::net::TcpStream;
use std::path::PathBuf;
use std::path::Path;
use crate::io::leer_a_string;
use crate::utilidades_de_compresion;
use crate::tipos_de_dato::comandos::log::Log;
use std::collections::HashMap;
use std::collections::HashSet;
use crate::tipos_de_dato::comandos::write_tree;
use crate::tipos_de_dato::objetos::tree::Tree;
// idea: Key -> (String, String) , primera entrada la ref que tiene el cliente, segunda la que tiene el sv.
pub struct Push { 
    hash_refs: HashMap<String, (String, String)>
}

impl Push { 
    pub fn new() -> Self {
        let mut hash_refs: HashMap<String, (String, String)> = HashMap::new();
        let refs = obtener_refs_de(PathBuf::from("./.gir/refs/"), String::from("./.gir/"));
        for referencia in refs {
            hash_refs.insert(
                referencia.split(' ').collect::<Vec<&str>>()[1].to_string(),
                (referencia.split(' ').collect::<Vec<&str>>()[0].to_string(), "0".repeat(40)),
            );
        }        
        Push { hash_refs }
    }

    pub fn ejecutar(&mut self) -> Result<String, String> { 
        println!("Se ejecuto el comando push");
        let server_address = "127.0.0.1:9418"; // Cambia la dirección IP si es necesario

        let mut client = TcpStream::connect(server_address).unwrap();
        let mut comunicacion = Comunicacion::new(client.try_clone().unwrap());
        let request_data = "git-receive-pack /gir/\0host=example.com\0\0version=1\0"; //en donde dice /.git/ va la dir del repo

        let request_data_con_largo_hex = io::obtener_linea_con_largo_hex(request_data);

        client.write_all(request_data_con_largo_hex.as_bytes()).unwrap();
        let version = comunicacion.aceptar_pedido().unwrap(); //cambiar aceptar pedido a obtener_linea

        let mut refs_recibidas = comunicacion.obtener_lineas().unwrap();
        println!("refs recibidas: {:?}", refs_recibidas);
        let mut actualizaciones = Vec::new();
        let mut objetos_a_enviar  = HashSet::new();

        let first_ref = refs_recibidas.remove(0);
        // caso en el que el server no tiene refs 
        if !first_ref.contains(&"0".repeat(40)){ 
            let referencia_y_capacidades = first_ref.split('\0').collect::<Vec<&str>>();
            let capabilities = referencia_y_capacidades[1];
            // println!("referencia_y_capacidades: {:?}", referencia_y_capacidades);
            refs_recibidas.push(referencia_y_capacidades[0].to_string());
        } 
        for referencia in &refs_recibidas {
            let obj_id = referencia.split(' ').collect::<Vec<&str>>()[0];
            let referencia = referencia.split(' ').collect::<Vec<&str>>()[1];
            match self.hash_refs.get_mut(referencia) { 
                Some(hash) => { 
                    println!("Entre aca para la ref: {}", referencia);
                    hash.1 = obj_id.to_string();
                    // if hash.0 != obj_id { 
                    //     hash.1 = obj_id.to_string();
                    // } else {
                    //     // borra la entrada (ver esta parte..)
                    //     hash.1 = 
                    //     // self.hash_refs.remove_entry(referencia);
                    // }
                }   
                None => {
                    // el server tiene un head que el cliente no tiene, abortar push (no borramos brancahes por lo tanto el sv esta por delante)
                }
                
            }
        }
        println!("hash_refs: {:?}", self.hash_refs);
   
        for (key, value) in &self.hash_refs {
            actualizaciones.push(io::obtener_linea_con_largo_hex(&format!("{} {} {}", &value.1, &value.0, &key))); // viejo (el del sv), nuevo (cliente), ref
            // checkear que no existan los objetos antes de appendear
            println!("value del cliente: {}, value del server: {}", value.0, value.1);
            if value.1 != value.0 {
                objetos_a_enviar.extend(obtener_commits_y_objetos_asociados(&key, &value.1).unwrap());
            } 
            if value.1 == "0".repeat(40) {
                objetos_a_enviar.extend(obtener_commits_y_objetos_asociados(&key, &value.0).unwrap());
            }
            // else {
                   // objetos_a_enviar.extend(obtener_commits_y_objetos_asociados(&key, &value.0).unwrap());
                
            // }
        }   
        println!("objetos a enviar: {:?}", objetos_a_enviar);

        if !actualizaciones.is_empty(){
            comunicacion.responder(actualizaciones).unwrap();
            comunicacion.responder_con_bytes(Packfile::new().obtener_pack_con_archivos(objetos_a_enviar.into_iter().collect(), "./.gir/objects/")).unwrap();            
            Ok(String::from("Push ejecutado con exito"))
        } else {
            //error 
            return Err("No hay actualizaciones".to_string());
        }

        // println!("Refs recibidas: {:?}", refs_recibidas);
    }
  
}

// funcion que aniade commits y sus objetos asociados hasta llegar al commit limite (criterio de parada)
fn obtener_commits_y_objetos_asociados(referencia: &String, commit_limite: &String) -> Result<HashSet<String>, String> {
    println!("Entro para la referencia {} y el commit limite: {}", referencia, commit_limite);
    let ruta = format!(".gir/{}", referencia);
    let mut ultimo_commit = leer_a_string(Path::new(&ruta))?;
    if ultimo_commit.is_empty() {
        return Ok(HashSet::new());
    }   
    let mut historial_commits: HashSet<String> = HashSet::new();
   
    loop {
        // obtengo el hash del tree que guarda el commit, le pido los objetos y los guardo en el set
        let hash_tree = write_tree::conseguir_arbol_padre_from_ult_commit_de_dir(&ultimo_commit, "./.gir/objects/".to_string()); 
        println!("Consiguiendo los objetos para el hash tree: {}", hash_tree);
        let tree = Tree::from_hash(hash_tree.clone(), PathBuf::from("./.gir/objects/"))?;
        historial_commits.insert(hash_tree);
        historial_commits.extend(tree.obtener_objetos().iter().map(|objeto| objeto.obtener_hash()));
        // historial_commits.extend(tree.objetos.iter().map(|objeto| objeto.obtener_hash()));
        // obtengo el padre del commit 
        let contenido = utilidades_de_compresion::descomprimir_objeto(ultimo_commit.clone(), String::from("./.gir/objects/"))?;
        let siguiente_padre = Log::conseguir_padre_desde_contenido_commit(&contenido);
        historial_commits.insert(ultimo_commit.clone());
        if siguiente_padre.is_empty() || siguiente_padre == commit_limite.to_string() {
            break;
        }
        ultimo_commit = siguiente_padre.to_string();
        println!("Siguiente padre: {}", siguiente_padre);
    }
    Ok(historial_commits)
}

fn obtener_refs_de(dir: PathBuf, prefijo: String) -> Vec<String> {
    let mut refs: Vec<String> = Vec::new();
    refs.append(&mut io::obtener_refs(dir.join("heads/"), prefijo.clone()).unwrap());
    refs.append(&mut io::obtener_refs(dir.join("tags/"), prefijo).unwrap());
    // refs[0] = self.agregar_capacidades(refs[0].clone ());
    refs
}





// juani@Juani:~/23C2-Cangrejos-Tacticos (servidor)$ rm -rf ./srv/gir/objects/
// juani@Juani:~/23C2-Cangrejos-Tacticos (servidor)$ rm ./srv/gir/refs/heads/master
// juani@Juani:~/23C2-Cangrejos-Tacticos (servidor)$ rm -rf ./srv/gir/objects/
// juani@Juani:~/23C2-Cangrejos-Tacticos (servidor)$ cp -r ../.gir/refs/heads/master ./srv/gir/refs/heads/
// juani@Juani:~/23C2-Cangrejos-Tacticos (servidor)$ cp -r ../.gir/objects/ ./srv/gir/objects/

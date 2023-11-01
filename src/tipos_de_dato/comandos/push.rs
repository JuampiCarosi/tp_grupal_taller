use crate::packfile::Packfile;
use crate::{comunicacion::Comunicacion, io};
use std::io::Write;
use std::path::PathBuf;
use std::path::Path;
use std::net::TcpStream;
use crate::io::leer_a_string;
use crate::utilidades_de_compresion;
use crate::tipos_de_dato::comandos::log::Log;
use std::collections::HashMap;
use std::collections::HashSet;


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
                (referencia.split(' ').collect::<Vec<&str>>()[0].to_string(), "0".repeat(20)),
            );
        }        
        println!("hash refs: {:?}", hash_refs)  ;
        Push { hash_refs }
    }
    pub fn ejecutar(&mut self) -> Result<String, String> { 
        println!("Se ejecuto el comando push");
        let server_address = "127.0.0.1:9418"; // Cambia la dirección IP si es necesario

        let mut client = TcpStream::connect(server_address).unwrap();
        let mut comunicacion = Comunicacion::new(client.try_clone().unwrap());

        // si es un push, tengo que calcular los commits de diferencia entre el cliente y el server, y mandarlos como packfiles.
        // hay una funcion que hace el calculo 
        // obtener_listas_de_commits
        let request_data = "git-receive-pack /.gir/\0host=example.com\0\0version=1\0"; //en donde dice /.git/ va la dir del repo
        let request_data_con_largo_hex = io::obtener_linea_con_largo_hex(request_data);

        client.write_all(request_data_con_largo_hex.as_bytes()).unwrap();
        let mut refs_recibidas = comunicacion.obtener_lineas().unwrap();
        if !refs_recibidas.is_empty() {
            let first_ref = refs_recibidas.remove(0);
        }

        for referencia in &refs_recibidas {
            let obj_id = referencia.split(' ').collect::<Vec<&str>>()[0];
            let referencia = referencia.split(' ').collect::<Vec<&str>>()[1];
            match self.hash_refs.get_mut(referencia) { 
                Some(hash) => { 
                    if hash.0 != obj_id { 
                        hash.1 = obj_id.to_string();
                    } else {
                        // borra la entrada (ver esta parte..)
                        self.hash_refs.remove_entry(referencia);
                    }
                }   
                None => {
                    // el server tiene un head que el cliente no tiene, abortar push (no borramos brancahes por lo tanto el sv esta)
                }

            }
        }
        // falta contemplar creado y borrado
        // cuando es update se manda viejo, nuevo, ref
        let mut actualizaciones = Vec::new();
        let mut objetos_a_enviar  = HashSet::new();
        for (key, value) in &self.hash_refs {
            actualizaciones.push(io::obtener_linea_con_largo_hex(&format!("{} {} {}\n", value.1, value.0, key))); // viejo (el del sv), nuevo (cliente), ref
            // checkear que no existan los objetos antes de appendear
            objetos_a_enviar.extend(obtener_listas_de_commits(key, &value.1).unwrap());
        }
        println!("actualizaciones: {:?}", actualizaciones);
        if !actualizaciones.is_empty(){
            comunicacion.responder(actualizaciones).unwrap();
            comunicacion.responder_con_bytes(Packfile::new().obtener_pack_con_archivos(objetos_a_enviar.into_iter().collect(), "./.gir/objects")).unwrap();            
            Ok(String::from("Push ejecutado con exito"))
        } else {
            //error 
            return Err("No hay actualizaciones".to_string());
        }

        // println!("Refs recibidas: {:?}", refs_recibidas);
    }
  
}

fn obtener_listas_de_commits(referencia: &String, commit_limite: &String) -> Result<Vec<String>, String> {
    let ruta = format!(".gir/{}", referencia);
    let mut ultimo_commit = leer_a_string(Path::new(&ruta))?;
    if ultimo_commit.is_empty() {
        return Ok(Vec::new());
    }   
    let mut historial_commits: Vec<String> = Vec::new();
    loop {
        let contenido = utilidades_de_compresion::descomprimir_objeto(ultimo_commit.clone(), String::from("./.gir/objects"))?;
        let siguiente_padre = Log::conseguir_padre_desde_contenido_commit(&contenido);
        historial_commits.push(ultimo_commit.clone());
        if siguiente_padre.is_empty() || siguiente_padre == commit_limite.to_string() {
            break;
        }
        ultimo_commit = siguiente_padre.to_string();
    }
    Ok(historial_commits)
}

fn obtener_refs_de(dir: PathBuf, prefijo: String) -> Vec<String> {
    let mut refs: Vec<String> = Vec::new();
    refs.append(&mut io::obtener_refs(dir.join("heads/"), prefijo.clone()).unwrap());
    refs.append(&mut io::obtener_refs(dir.join("tags/"), prefijo).unwrap());
    // refs[0] = self.agregar_capacidades(refs[0].clone());
    refs
}






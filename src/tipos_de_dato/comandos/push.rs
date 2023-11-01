use crate::io::{escribir_bytes, obtener_archivos_faltantes};
use crate::{comunicacion::Comunicacion, io, packfile, tipos_de_dato::objetos::tree::Tree, tipos_de_dato::comandos::write_tree};
use std::io::Write;
use std::path::PathBuf;
use std::path::Path;
use std::rc::Rc;
use std::net::TcpStream;
use crate::io::leer_a_string;
use crate::utilidades_de_compresion;
use crate::tipos_de_dato::comandos::log::Log;
pub struct Push { 
    refs: Vec<String>
}
impl Push { 
    pub fn new() -> Self {
        let refs = obtener_refs_de(PathBuf::from("./.gir/refs/"), String::from("./.gir/"));
        println!("Refs: {:?}", refs);
        Push { refs }
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
        let first_ref = refs_recibidas.remove(0);

        // pasos a seguir: 
        // 1) obtener los commits que no estan en el server (esto se hace comparando los hashes de las refs recibidas con los locales)
        // 2) especificar cuales refs tienen nuevos commits como referencia para que el server lo actualice
        // 3) enviar los objetos que no estan en el server como packfiles (para eso usar la funcion de mateo)
        // 4) en algun lugar hay que checkear que no se modifica el repositorio mientras ocurre esta negociacion, en cuyo caso se debe abortar el push
    
        // voy a suponer el caso base, que es cuando hay que mandar actualizaciones al servidor
        let mut lista: Vec<String> = Vec::new();
        // for referencia in refs_recibidas {
        //     let obj_id = referencia.split(' ').collect::<Vec<&str>>()[0];
        //     let referencia = referencia.split('/').collect::<Vec<&str>>();
        //     // checkear que branchs hay en el local que no haya obtenido como referencias, tambien las que me paso el server y no tengo 
        //     // En dicho checkear si error o si delete
        //     let lista = obtener_listas_de_commits(&referencia.last().unwrap().to_string());


        // }

        // println!("Refs recibidas: {:?}", refs_recibidas);


        let archivos_faltantes = obtener_archivos_faltantes(refs_recibidas, "./.gir/refs".to_string());

        // println!("Refs recibidas: {:?}", refs_recibidas);
        Ok(String::from("Push ejecutado con exito"))
    }
  
}

fn obtener_listas_de_commits(branch: &String) -> Result<Vec<String>, String> {
    let ruta = format!(".gir/refs/heads/{}", branch);
    let mut ultimo_commit = leer_a_string(Path::new(&ruta))?;

    if ultimo_commit.is_empty() {
        return Ok(Vec::new());
    }   
    let mut historial_commits: Vec<String> = Vec::new();
    loop {
        let contenido = utilidades_de_compresion::descomprimir_objeto(ultimo_commit.clone(), ruta.clone())?;
        let siguiente_padre = Log::conseguir_padre_desde_contenido_commit(&contenido);
        historial_commits.push(ultimo_commit.clone());
        if siguiente_padre.is_empty() {
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
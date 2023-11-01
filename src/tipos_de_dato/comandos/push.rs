use crate::io::leer_a_string;
use crate::io::{escribir_bytes, obtener_archivos_faltantes};
use crate::tipos_de_dato::logger::Log;
use crate::utilidades_de_compresion;
use crate::{
    comunicacion::Comunicacion, io, packfile, tipos_de_dato::comandos::write_tree,
    tipos_de_dato::objetos::tree::Tree,
};
use std::io::Write;
use std::net::TcpStream;
use std::path::PathBuf;
use std::rc::Rc;
pub struct Push;
impl Push {
    pub fn new() -> Self {
        Push
    }
    pub fn ejecutar(&mut self) -> Result<String, String> {
        let server_address = "127.0.0.1:9418"; // Cambia la dirección IP si es necesario
        let mut comunicacion = Comunicacion::new_desde_direccion_servidor(server_address)?;

        // si es un push, tengo que calcular los commits de diferencia entre el cliente y el server, y mandarlos como packfiles.
        // hay una funcion que hace el calculo
        // obtener_listas_de_commits
        let request_data = "git-receive-pack /.gir/\0host=example.com\0\0version=1\0"; //en donde dice /.git/ va la dir del repo
        let request_data_con_largo_hex = io::obtener_linea_con_largo_hex(request_data);
        comunicacion.enviar(&request_data_con_largo_hex)?;
        
        let mut refs_recibidas = comunicacion.obtener_lineas().unwrap();
        let first_ref = refs_recibidas.remove(0);

        // pasos a seguir:
        // 1) obtener los commits que no estan en el server (esto se hace comparando los hashes de las refs recibidas con los locales)
        // 2) especificar cuales refs tienen nuevos commits como referencia para que el server lo actualice
        // 3) enviar los objetos que no estan en el server como packfiles (para eso usar la funcion de mateo)
        // 4) en algun lugar hay que checkear que no se modifica el repositorio mientras ocurre esta negociacion, en cuyo caso se debe abortar el push

        // let archivos_faltantes = obtener_archivos_faltantes(refs_recibidas, "./.gir/refs".to_string());

        // println!("Refs recibidas: {:?}", refs_recibidas);
        Ok(String::from("Push ejecutado con exito"))
    }
}

// fn obtener_listas_de_commits(branch: &String) -> Result<Vec<String>, String> {
//     let ruta = format!(".gir/refs/heads/{}", branch);
//     let mut ultimo_commit = leer_a_string(path::Path::new(&ruta))?;

//     if ultimo_commit.is_empty() {
//         return Ok(Vec::new());
//     }
//     let mut historial_commits: Vec<String> = Vec::new();
//     loop {
//         let contenido = utilidades_de_compresion::descomprimir_objeto(ultimo_commit.clone())?;
//         let siguiente_padre = Log::conseguir_padre_desde_contenido_commit(&contenido);
//         historial_commits.push(ultimo_commit.clone());
//         if siguiente_padre.is_empty() {
//             break;
//         }
//         ultimo_commit = siguiente_padre.to_string();
//     }
//     Ok(historial_commits)
// }

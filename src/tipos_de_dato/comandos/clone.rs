use crate::io::escribir_bytes;
use crate::{comunicacion::Comunicacion, io, packfile, tipos_de_dato::objetos::tree::Tree, tipos_de_dato::comandos::write_tree};
use std::io::Write;
use std::path::PathBuf;
use std::rc::Rc;
use std::{char::decode_utf16, net::TcpStream};
// use gir::tipos_de_dato::{comando::Comando, logger::Logger};


//-------- ATENCION ----------
// Si hay una ref que no apunta a nada porque esta vacia, rompe al hacer el split de refs.

pub struct Clone;

impl Clone { 
    pub fn new() -> Self{ 
        Clone{}
    }

    pub fn ejecutar(&mut self) -> Result<String, String> {

        let server_address = "127.0.0.1:9418"; // Cambia la dirección IP si es necesario
        let mut comunicacion = Comunicacion::new_desde_direccion_servidor(server_address)?;

        // si es un push, tengo que calcular los commits de diferencia entre el cliente y el server, y mandarlos como packfiles.
        // hay una funcion que hace el calculo 
        // obtener_listas_de_commits
        let request_data = "git-upload-pack /.gir/\0host=example.com\0\0version=1\0"; //en donde dice /.git/ va la dir del repo
        let request_data_con_largo_hex = io::obtener_linea_con_largo_hex(request_data);
        comunicacion.enviar(&request_data_con_largo_hex)?;
        
        let refs_recibidas = comunicacion.obtener_lineas().unwrap();
        
        for referencia in &refs_recibidas { 
            let referencia_y_contenido = referencia.split_whitespace().collect::<Vec<&str>>();
            if !&referencia_y_contenido[1].contains("HEAD"){
                let dir = PathBuf::from("./.gir/".to_string() + referencia_y_contenido[1]);
                println!("Voy a escribir en: {:?}", dir);
                escribir_bytes(dir, referencia_y_contenido[0]).unwrap();
            }
        }   
        let capacidades = refs_recibidas[0].split("\0").collect::<Vec<&str>>()[1];
        let wants = comunicacion.obtener_wants_pkt(&refs_recibidas, capacidades.to_string()).unwrap();
        comunicacion.responder(wants.clone()).unwrap();
        // Esto porque es un CLONE
        comunicacion.responder(vec![io::obtener_linea_con_largo_hex("done")]).unwrap();
        let acks_nak = comunicacion.obtener_lineas().unwrap();
        println!("acks_nack: {:?}", acks_nak);

        println!("Obteniendo paquete..");
        let mut packfile = comunicacion.obtener_lineas_como_bytes().unwrap();
        comunicacion.obtener_paquete_y_escribir(&mut packfile, String::from("./.gir/objects/")).unwrap();

        let head_dir: String = String::from(".gir/HEAD");
        
        let ref_head: String = if refs_recibidas[0].contains("HEAD") {
            refs_recibidas[0].split("\0").collect::<Vec<&str>>()[0].to_string().split(" ").collect::<Vec<&str>>()[0].to_string()
        } else {
            refs_recibidas[0].split("\0").collect::<Vec<&str>>()[0].to_string()
        };
        io::escribir_bytes(PathBuf::from(head_dir), b"refs/heads/master").unwrap();
        // println!("ref_head: {:?}", ref_head);
        
        let tree_hash = write_tree::conseguir_arbol_padre_from_ult_commit_de_dir(ref_head.clone(), String::from("./.gir/objects/"));
        println!("tree_hash: {:?}", tree_hash);

        let tree: Tree = Tree::from_hash(tree_hash, PathBuf::from(env!("CARGO_MANIFEST_DIR").to_string() + "/.gir/objects")).unwrap();
        match tree.escribir_en_directorio() {
            Ok(_) => {},
            Err(e) => {println!("Error al escribir el arbol: {}", e);}
        }


        Ok(String::from("Clone ejecutado con exito"))
    }
}
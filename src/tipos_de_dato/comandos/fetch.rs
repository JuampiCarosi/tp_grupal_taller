use crate::io::escribir_bytes;
use crate::packfile::Packfile;
use crate::{comunicacion::Comunicacion, io};
use std::io::Write;
use std::path::PathBuf;
use std::net::TcpStream;


pub struct Fetch;

impl Fetch {
    pub fn new() -> Self{
        Fetch{}
    }

    pub fn ejecutar(&mut self) -> Result<String, String> {
        println!("Se ejecutó el comando fetch");
        // esto deberia llamar a fetch-pack
        // let server_address = "127.0.0.1:9418"; // hardcodeado
        let mut client = TcpStream::connect(("localhost", 9418)).unwrap();
        let mut comunicacion = Comunicacion::new(client.try_clone().unwrap());

        // si es un push, tengo que calcular los commits de diferencia entre el cliente y el server, y mandarlos como packfiles.
        // hay una funcion que hace el calculo
        // obtener_listas_de_commits
        // ===============================================================================
        // EN LUGAR DE GIR HAY QUE PONER EL NOMBRE DE LA CARPETA QUE LO CONTIENE
        // ===============================================================================
        let request_data = "git-upload-pack /gir/\0host=example.com\0\0version=1\0"; //en donde dice /.git/ va la dir del repo
        let request_data_con_largo_hex = io::obtener_linea_con_largo_hex(request_data);

        client.write_all(request_data_con_largo_hex.as_bytes()).unwrap();
        let mut refs_recibidas = comunicacion.obtener_lineas().unwrap();

        if refs_recibidas.len() == 1 {
            return Ok(String::from("No hay refs"));
        }
        println!("refs: {:?}", refs_recibidas);

        if refs_recibidas.is_empty() {
            return Err(String::from("No se recibieron referencias"));
        }
        let version = refs_recibidas.remove(0);
        let first_ref = refs_recibidas.remove(0);
        let referencia_y_capacidades = first_ref.split('\0').collect::<Vec<&str>>();
        let capacidades = referencia_y_capacidades[1];
        let diferencias = io::obtener_diferencias_remote(refs_recibidas, "./.gir/".to_string());
        if diferencias.is_empty(){
            comunicacion.enviar_flush_pkt().unwrap();
            return Ok(String::from("El cliente esta actualizado"));
        }
        let wants = comunicacion.obtener_wants_pkt(&diferencias, "".to_string()).unwrap();
        println!("wants: {:?}", wants);
        comunicacion.responder(wants.clone()).unwrap();

        let objetos_directorio = io::obtener_objetos_del_directorio("./.gir/objects/".to_string()).unwrap();

        let haves = comunicacion.obtener_haves_pkt(&objetos_directorio);
        if !haves.is_empty() {
            println!("Haves: {:?}", haves);
            comunicacion.responder(haves).unwrap();
            let acks_nak = comunicacion.obtener_lineas().unwrap();
            comunicacion.responder(vec![io::obtener_linea_con_largo_hex("done\n")]).unwrap();
            // let acks_nak = comunicacion.obtener_lineas().unwrap();
            println!("acks_nack: {:?}", acks_nak);
        } else {
            comunicacion.responder(vec![io::obtener_linea_con_largo_hex("done\n")]).unwrap();
            let acks_nak = comunicacion.obtener_lineas().unwrap();
            println!("acks_nack: {:?}", acks_nak);

        }
        
        println!("Obteniendo paquete..");
        let mut packfile = comunicacion.obtener_lineas_como_bytes().unwrap();
        Packfile::new().obtener_paquete_y_escribir(&mut packfile, String::from("./.gir/objects/")).unwrap();
        escribir_en_remote_origin_las_referencias(&diferencias);

        Ok(String::from("Fetch ejecutado con exito"))
    }
}

fn escribir_en_remote_origin_las_referencias(referencias: &Vec<String>) {
    let remote_origin = "./.gir/refs/remotes/origin/";

    for referencia in referencias {
        let referencia_y_contenido = referencia.split_whitespace().collect::<Vec<&str>>();
        let referencia_con_remote_origin = PathBuf::from(referencia_y_contenido[1]);
        let nombre_referencia = referencia_con_remote_origin.file_name().unwrap();
        let dir = PathBuf::from(remote_origin.to_string() + nombre_referencia.to_str().unwrap());
        println!("Voy a escribir en: {:?}", dir);
        escribir_bytes(dir, referencia_y_contenido[0]).unwrap();

    }


}
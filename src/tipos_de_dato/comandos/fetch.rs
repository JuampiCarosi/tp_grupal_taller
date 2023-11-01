use crate::io::escribir_bytes;
use crate::{
    comunicacion::Comunicacion, io, packfile, tipos_de_dato::comandos::write_tree,
    tipos_de_dato::objetos::tree::Tree,
};
use std::io::Write;
use std::net::TcpStream;
use std::path::PathBuf;
use std::rc::Rc;

pub struct Fetch;

impl Fetch {
    pub fn new() -> Self {
        Fetch {}
    }

    pub fn ejecutar(&mut self) -> Result<String, String> {
        let direccion_servidor = "127.0.0.1:9418"; // Cambia la dirección IP si es necesario
                                                   //se inicia la comunicacon con servidor
        let mut comunicacion = Comunicacion::new_desde_direccion_servidor(direccion_servidor)?;

        //Iniciar la comunicacion con el servidor
        // obtener_listas_de_commits
        let request_data = "git-upload-pack /.gir/\0host=example.com\0\0version=1\0"; //en donde dice /.git/ va la dir del repo
        let request_data_con_largo_hex = io::obtener_linea_con_largo_hex(request_data);
        comunicacion.enviar(&request_data_con_largo_hex)?;

        let mut refs_recibidas = comunicacion.obtener_lineas().map_err(|e| {
            format!(
                "Fallo en la lectura de los contendios del servidor.\n{:?}\n",
                e
            )
        })?;
        let first_ref = refs_recibidas.remove(0);
        escribir_en_remote_origin_las_referencias(&refs_recibidas);

        let capacidades = first_ref.split("\0").collect::<Vec<&str>>()[1];
        // envio
        let wants = comunicacion
            .obtener_wants_pkt(&refs_recibidas, capacidades.to_string())
            .unwrap();
        comunicacion.responder(wants.clone()).unwrap();

        let objetos_directorio =
            io::obtener_objetos_del_directorio("./.gir/objects/".to_string()).unwrap();
        let haves = comunicacion.obtener_haves_pkt(&objetos_directorio);
        if !haves.is_empty() {
            comunicacion.responder(haves).unwrap();
            let acks_nak = comunicacion.obtener_lineas().unwrap();
            println!("acks_nack: {:?}", acks_nak);
            comunicacion
                .responder(vec![io::obtener_linea_con_largo_hex("done")])
                .unwrap();
        } else {
            comunicacion
                .responder(vec![io::obtener_linea_con_largo_hex("done")])
                .unwrap();
            let acks_nak = comunicacion.obtener_lineas().unwrap();
            println!("acks_nack: {:?}", acks_nak);
        }

        // aca para git daemon hay que poner un recibir linea mas porque envia un ACK repetido (No entiendo por que...)
        println!("Obteniendo paquete..");
        let mut packfile = comunicacion.obtener_lineas_como_bytes().unwrap();
        comunicacion
            .obtener_paquete_y_escribir(&mut packfile, String::from("./.gir/objects/"))
            .unwrap();
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

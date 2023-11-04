use std::io::Write;
use std::io::Read;
use std::net::TcpStream;

use crate::err_comunicacion::ErrorDeComunicacion;
use crate::io::obtener_archivos_faltantes;
use crate::packfile;
use crate::{comunicacion::Comunicacion, io as git_io};
use crate::utilidades_strings;

pub fn upload_pack(dir: String, comunicacion: &mut Comunicacion<TcpStream>) -> Result<(), ErrorDeComunicacion> {
    // caso push 
    // let refs_a_actualizar = comunicacion.obtener_lineas().unwrap();
    let wants = comunicacion.obtener_lineas().unwrap(); // obtengo los wants del cliente
    if wants.is_empty() {
        println!("Se termino la conexion");
        return Ok(()); // el cliente esta actualizado
    }
    // ------- CLONE --------
    // a partir de aca se asume que va a ser un clone porque es el caso mas sencillo, despues cambiar
    let lineas_siguientes = comunicacion.obtener_lineas().unwrap();
    // println!("Lineas siguientes: {:?}", lineas_siguientes);
    if lineas_siguientes[0].clone().contains("done") {
        comunicacion.responder(vec![git_io::obtener_linea_con_largo_hex("NAK\n")])?; // respondo NAK
        // let want_obj_ids = utilidades_strings::eliminar_prefijos(&mut wants, "want");
        // println!("want_obj_ids: {:?}", want_obj_ids);
        let packfile =
            packfile::Packfile::new().obtener_pack_entero(&(dir.clone().to_string() + "objects/")); // obtengo el packfile
            // git_io::leer_bytes("./.git/objects/pack/pack-31897a1f902980a7e540e812b54f5702f449af8b.pack").unwrap();
        comunicacion.responder_con_bytes(packfile).unwrap();
        println!("Upload pack ejecutado con exito");
        return Ok(());
    }

    // -------- fetch ----------
    println!("Entro aca porque hay haves");
    let have_objs_ids = utilidades_strings::eliminar_prefijos(&lineas_siguientes);
    println!("have_objs_ids: {:?}", have_objs_ids);
    // let have_obj_ids = utilidades_strings::eliminar_prefijos(&mut lineas_siguientes, "have");
    let respuesta_acks_nak = git_io::obtener_ack(have_objs_ids.clone(), dir.clone() + "objects/");
    println!("respuesta_acks_nak: {:?}", respuesta_acks_nak);
    comunicacion.responder(respuesta_acks_nak).unwrap();
    // let lineas = comunicacion.obtener_lineas().unwrap();
    // println!("lineas: {:?}", lineas);
    let faltantes = obtener_archivos_faltantes(have_objs_ids, dir.clone());
    // obtener un packfile de los faltantes...
    let packfile = packfile::Packfile::new().obtener_pack_con_archivos(faltantes, &(dir.clone() + "objects/"));
    comunicacion.responder_con_bytes(packfile).unwrap();
    println!("Upload pack ejecutado con exito");
    Ok(())
}
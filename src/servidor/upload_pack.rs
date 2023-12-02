use std::net::TcpStream;
use crate::tipos_de_dato::comunicacion::Comunicacion;
use crate::tipos_de_dato::packfile;
use crate::utils::io as git_io;
use crate::utils::strings::eliminar_prefijos;

pub fn upload_pack(
    dir: String,
    comunicacion: &mut Comunicacion<TcpStream>,
) -> Result<(), String> {
    // caso push
    // let refs_a_actualizar = comunicacion.obtener_lineas().unwrap();
    let wants = comunicacion.obtener_lineas()?; // obtengo los wants del cliente
    if wants.is_empty() {
        println!("Se termino la conexion");
        return Ok(()); // el cliente esta actualizado
    }
    // ------- CLONE --------
    // a partir de aca se asume que va a ser un clone porque es el caso mas sencillo, despues cambiar
    let lineas_siguientes = comunicacion.obtener_lineas()?;
    if lineas_siguientes[0].clone().contains("done") {
        comunicacion.responder(vec![git_io::obtener_linea_con_largo_hex("NAK\n")])?; // respondo NAK
                                                                                     // let want_obj_ids = utilidades_strings::eliminar_prefijos(&mut wants, "want");
                                                                                     // println!("want_obj_ids: {:?}", want_obj_ids);
        let packfile =
            packfile::Packfile::obtener_pack_entero(&(dir.clone().to_string() + "objects/"))?; // obtengo el packfile
                                                                                                    // git_io::leer_bytes("./.git/objects/pack/pack-31897a1f902980a7e540e812b54f5702f449af8b.pack").unwrap();
        comunicacion.enviar_pack_file(packfile)?;

        println!("Upload pack ejecutado con exito");
        return Ok(());
    }

    // -------- fetch ----------
    let have_objs_ids = eliminar_prefijos(&lineas_siguientes);
    // let have_obj_ids = utilidades_strings::eliminar_prefijos(&mut lineas_siguientes, "have");
    let respuesta_acks_nak =
        git_io::obtener_ack(have_objs_ids.clone(), &(dir.clone() + "objects/"));
    comunicacion.responder(respuesta_acks_nak)?;
    let _ultimo_done = comunicacion.obtener_lineas()?;
    let faltantes = git_io::obtener_archivos_faltantes(have_objs_ids, dir.clone());
    // obtener un packfile de los faltantes...
    let packfile =
        packfile::Packfile::obtener_pack_con_archivos(faltantes, &(dir.clone() + "objects/"))?;

    comunicacion.enviar_pack_file(packfile)?;
    println!("Upload pack ejecutado con exito");
    Ok(())
}

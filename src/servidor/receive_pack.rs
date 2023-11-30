use crate::err_comunicacion::ErrorDeComunicacion;

use crate::tipos_de_dato::comunicacion::Comunicacion;
use crate::tipos_de_dato::packfile;
use crate::utils::io;
use std::net::TcpStream;

pub fn receive_pack(
    dir: String,
    comunicacion: &mut Comunicacion<TcpStream>,
) -> Result<(), ErrorDeComunicacion> {
    println!("Se ejecuto el comando receive-pack");
    let actualizaciones = comunicacion.obtener_lineas().unwrap();
    let packfile = comunicacion.obtener_packfile().unwrap();
    // Packfile::new().obtener_paquete_y_escribir(&mut packfile, dir.clone() + "/gir/objects/")?; // uso otra convencion (/)por como esta hecho en daemon
    // las refs se actualizan al final
    packfile::leer_packfile_y_escribir(&packfile, &(dir.clone() + "objects/"))?;

    for actualizacion in &actualizaciones {
        let mut partes = actualizacion.split(' ');
        let vieja_ref = partes.next().unwrap_or("");
        let nueva_ref = partes.next().unwrap_or("");
        let referencia = partes.next().unwrap_or("");
        if nueva_ref != vieja_ref {
            io::escribir_bytes(dir.clone() + referencia, nueva_ref).unwrap();
        }
    }
    println!("Receive pack ejecutado con exito");
    Ok(())
}

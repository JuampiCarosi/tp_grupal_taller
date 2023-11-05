use crate::err_comunicacion::ErrorDeComunicacion;

use crate::packfile::Packfile;

use crate::comunicacion::Comunicacion;
use crate::utils::io;
use std::net::TcpStream;
use std::path::PathBuf;

pub fn receive_pack(
    dir: String,
    comunicacion: &mut Comunicacion<TcpStream>,
) -> Result<(), ErrorDeComunicacion> {
    println!("Se ejecuto el comando receive-pack");
    let actualizaciones = comunicacion.obtener_lineas().unwrap();
    if actualizaciones.is_empty() {
        return Ok(());
    }
    println!("actualizaciones: {:?}", actualizaciones);
    let mut packfile = comunicacion.obtener_lineas_como_bytes().unwrap();
    Packfile::new().obtener_paquete_y_escribir(&mut packfile, dir.clone() + "/gir/objects/")?; // uso otra convencion (/)por como esta hecho en daemon

    // las refs se actualizan al final
    for actualizacion in &actualizaciones {
        let mut parts = actualizacion.splitn(2, ' ');
        let _vieja_ref = parts.next().unwrap_or("");
        let nueva_ref = parts.next().unwrap_or("");
        println!("Voy a escribir la referencia: {:?}", nueva_ref);
        io::escribir_referencia(nueva_ref, PathBuf::from(format!("{}/{}", dir, "gir")));
        // en donde dice dir va la dir del repo
    }
    println!("Receive pack ejecutado con exito");
    Ok(())
}

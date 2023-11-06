use crate::err_comunicacion::ErrorDeComunicacion;

use crate::tipos_de_dato::comunicacion::Comunicacion;
use crate::tipos_de_dato::packfile::Packfile;
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
    let mut packfile = comunicacion.obtener_packfile().unwrap();
    println!("packfile: {:?}", packfile);
    Packfile::new().obtener_paquete_y_escribir(&mut packfile, dir.clone() + "/gir/objects/")?; // uso otra convencion (/)por como esta hecho en daemon
    println!("Post pack");
    // las refs se actualizan al final
    for actualizacion in &actualizaciones {
        let mut partes = actualizacion.splitn(2, ' ');
        println!("Partes: {:?}", partes);
        let _vieja_ref = partes.next().unwrap_or("");
        let nueva_ref = partes.next().unwrap_or("");
        if nueva_ref != _vieja_ref {
            println!("Vieja ref: {:?}", _vieja_ref);
            println!("Voy a escribir la referencia: {:?}", nueva_ref);
            io::escribir_referencia(nueva_ref, PathBuf::from(format!("{}/{}", dir, "gir")));
        }
        // en donde dice dir va la dir del repo
    }
    println!("Receive pack ejecutado con exito");
    Ok(())
}

use crate::err_comunicacion::ErrorDeComunicacion;

use crate::tipos_de_dato::comunicacion::Comunicacion;
use crate::tipos_de_dato::packfile;
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
    let mut packfile = comunicacion.obtener_packfile().unwrap();
    // Packfile::new().obtener_paquete_y_escribir(&mut packfile, dir.clone() + "/gir/objects/")?; // uso otra convencion (/)por como esta hecho en daemon
                                                                                               // las refs se actualizan al final
    packfile::leer_packfile_y_escribir(&mut packfile, dir.clone() + "/gir/objects/")?;
   
    for actualizacion in &actualizaciones {
        let mut partes = actualizacion.splitn(2, ' ');
        let _vieja_ref = partes.next().unwrap_or("");
        let nueva_ref = partes.next().unwrap_or("");
        if nueva_ref != _vieja_ref {
            io::escribir_referencia(nueva_ref, PathBuf::from(format!("{}/{}", dir, "gir")));
        }
        // en donde dice dir va la dir del repo
    }
    println!("Receive pack ejecutado con exito");
    Ok(())
}

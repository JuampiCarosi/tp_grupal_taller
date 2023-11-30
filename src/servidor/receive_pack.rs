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
    let mut packfile = comunicacion.obtener_packfile().unwrap();

    packfile::leer_packfile_y_escribir(&mut packfile, dir.clone() + "objects/")?;

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

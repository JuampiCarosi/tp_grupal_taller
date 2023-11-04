use crate::err_comunicacion::ErrorDeComunicacion;
use crate::{comunicacion::Comunicacion, io};
use std::path::PathBuf;

pub fn receive_pack(
    dir: String,
    comunicacion: &mut Comunicacion,
) -> Result<(), ErrorDeComunicacion> {
    let actualizaciones = comunicacion.obtener_lineas().unwrap();
    if actualizaciones.is_empty() {
        return Ok(());
    }
    println!("actualizaciones: {:?}", actualizaciones);
    println!("direccion: {:?}", dir);
    for actualizacion in &actualizaciones {
        let mut parts = actualizacion.splitn(2, ' ');
        let vieja_ref = parts.next().unwrap_or("");
        let nueva_ref = parts.next().unwrap_or("");
        println!("Voy a escribir la referencia: {:?}", nueva_ref);
        io::escribir_referencia(nueva_ref, PathBuf::from(format!("{}/{}", dir, ".gir")));
    }
    let mut packfile = comunicacion.obtener_lineas_como_bytes().unwrap();
    comunicacion.obtener_paquete_y_escribir(&mut packfile, "./srv/.gir/objects/".to_string())?;

    println!("Receive pack ejecutado con exito");
    Ok(())
}

use crate::err_comunicacion::ErrorDeComunicacion;
use crate::io::obtener_archivos_faltantes;
use crate::{packfile, comunicacion};
use crate::{comunicacion::Comunicacion, io as git_io};
use crate::utilidades_strings;

pub fn receive_pack(refs: Vec<String>, dir: String, comunicacion: &mut Comunicacion) -> Result<(), ErrorDeComunicacion> {

    comunicacion.responder(refs)?; // respondo con las refs 
    let actualizaciones = comunicacion.obtener_lineas();
    println!("actualizaciones: {:?}", actualizaciones);
    

    println!("Receive pack ejecutado con exito");
    Ok(())
}
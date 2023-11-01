use crate::err_comunicacion::ErrorDeComunicacion;
use crate::io::obtener_archivos_faltantes;
use crate::{packfile, comunicacion};
use crate::{comunicacion::Comunicacion, io};
use crate::utilidades_strings;
use std::path::PathBuf;

pub fn receive_pack(refs: Vec<String>, dir: String, comunicacion: &mut Comunicacion) -> Result<(), ErrorDeComunicacion> {

    comunicacion.responder(refs)?; // respondo con las refs 
    let actualizaciones = comunicacion.obtener_lineas().unwrap();
    println!("actualizaciones: {:?}", actualizaciones);
    println!("direccion: {:?}", dir);
    for actualizacion in &actualizaciones { 
        let mut parts = actualizacion.splitn(2, ' ');
        let vieja_ref = parts.next().unwrap_or("");
        let nueva_ref = parts.next().unwrap_or("");
        println!("Voy a escribir la referencia: {:?}", nueva_ref);
        // io::escribir_referencia(nueva_ref, PathBuf::from(format!("{}/{}", dir, ".gir")));
        
    }  

    println!("Receive pack ejecutado con exito");
    Ok(())
}
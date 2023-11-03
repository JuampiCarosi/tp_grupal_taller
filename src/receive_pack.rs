
use std::io::Write;
use std::io::Read;

use crate::err_comunicacion::ErrorDeComunicacion;
use crate::comunicacion::Comunicacion;


pub fn receive_pack<T: Write + Read>(refs: Vec<String>, dir: String, comunicacion: &mut Comunicacion<T>) -> Result<(), ErrorDeComunicacion> {

    comunicacion.responder(refs)?; // respondo con las refs 
    



    println!("Receive pack ejecutado con exito");
    Ok(())
}
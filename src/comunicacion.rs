use crate::err_comunicacion::ErrorDeComunicacion;
use std::net::TcpStream;
use std::io::{Read, Write};
use std::str;
pub fn aceptar_pedido(flujo: &mut TcpStream) -> Result<String, ErrorDeComunicacion>{
    // lee primera parte, 4 bytes en hexadecimal indican el largo del stream
    let mut tamanio_bytes = [0; 4];
    flujo.read_exact(&mut tamanio_bytes)?;
    // largo de bytes a str
    let tamanio_str = str::from_utf8(&tamanio_bytes)?;
    // transforma str a u32
    let tamanio = u32::from_str_radix(tamanio_str, 16).unwrap();
    // lee el resto del flujo
    let mut data = vec![0; (tamanio - 4) as usize];
    flujo.read_exact(&mut data)?;
    let linea = str::from_utf8(&data)?;
    Ok(linea.to_string())
}

pub fn obtener_lineas(flujo: &mut TcpStream) -> Result<Vec<String>, ErrorDeComunicacion> {

    let mut lineas: Vec<String> = Vec::new();
    loop { 
        // lee primera parte, 4 bytes en hexadecimal indican el largo del stream
        let mut tamanio_bytes = [0; 4];
        flujo.read_exact(&mut tamanio_bytes)?;
        // largo de bytes a str
        let tamanio_str = str::from_utf8(&tamanio_bytes)?;
        // transforma str a u32
        let tamanio = u32::from_str_radix(tamanio_str, 16).unwrap();
        if tamanio == 0 {
            break;
        }
        // lee el resto del flujo
        let mut data = vec![0; (tamanio - 4) as usize];
        flujo.read_exact(&mut data)?;
        let linea = str::from_utf8(&data)?;
        lineas.push(linea.to_string());
    }
    println!("Received: {:?}", lineas);
    Ok(lineas)
}

pub fn responder(flujo: &mut TcpStream, lineas: Vec<String>) -> Result<(), ErrorDeComunicacion> {
    for linea in lineas { 
        flujo.write(linea.as_bytes())?;
    }
    flujo.write(String::from("0000").as_bytes())?;
    Ok(())
}

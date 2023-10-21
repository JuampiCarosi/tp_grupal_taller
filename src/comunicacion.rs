use crate::err_comunicacion::ErrorDeComunicacion;
use std::net::TcpStream;
use std::io::{Read, Write};
use std::str;
use crate::io;
    
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
        // println!("Received: {:?}", linea);
        lineas.push(linea.to_string());
    }
    // println!("Received: {:?}", lineas);
    Ok(lineas)
}

pub fn responder(flujo: &mut TcpStream, lineas: Vec<String>) -> Result<(), ErrorDeComunicacion> {
    for linea in lineas { 
        flujo.write(linea.as_bytes())?;
    }
    flujo.write(String::from("0000").as_bytes())?;
    Ok(())
}

pub fn responder_con_bytes(flujo: &mut TcpStream, lineas: Vec<Vec<u8>>) -> Result<(), ErrorDeComunicacion> {
    for linea in lineas { 
        flujo.write(&linea)?;
    }
    flujo.write(String::from("0000").as_bytes())?;
    Ok(())
}

pub fn obtener_obj_ids(lineas: &Vec<String>) -> Vec<String> {
    let mut obj_ids: Vec<String> = Vec::new();
    for linea in lineas {
        obj_ids.push(linea.split_whitespace().collect::<Vec<&str>>()[0].to_string());
    }
    obj_ids
}


pub fn obtener_wants(lineas: &Vec<String>, capacidades: String) -> Result<Vec<String>, ErrorDeComunicacion> {
    // hay que checkear que no haya repetidos, usar hashset
    let mut lista_wants: Vec<String> = Vec::new();
    let mut obj_ids = obtener_obj_ids(lineas);
    obj_ids[0].push_str(&(" ".to_string() + &capacidades));
    for linea in obj_ids {
        lista_wants.push(io::obtener_linea_con_largo_hex(&("want".to_string() + &linea)));     
    }
    Ok(lista_wants)
}

// pub fn obtener_capacidades(referencias: Vec<String>) -> Vec<&'static str> {
//     let capacidades = referencias[0].split("\0").collect::<Vec<&str>>().clone();
    
// }

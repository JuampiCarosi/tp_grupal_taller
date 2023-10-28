use std::{net::TcpStream, char::decode_utf16};
use std::io::Write;
use gir::{io, comunicacion::Comunicacion, packfile};
fn main() -> std::io::Result<()> {
    // Configura la dirección del servidor y el puerto
    let server_address = "127.0.0.1:9418"; // Cambia la dirección IP si es necesario

    // Conéctate al servidor
    let mut client = TcpStream::connect(server_address)?;
    let mut comunicacion = Comunicacion::new(client.try_clone().unwrap());
    // Envía datos al servidor (reemplaza esto por tus datos Git)
    let request_data = "git-upload-pack /.git/\0host=example.com\0\0version=1\0";
    let request_data_con_largo_hex = io::obtener_linea_con_largo_hex(request_data);
    
    client.write_all(request_data_con_largo_hex.as_bytes())?;
    
    // aca depende del comando, pero voy a hacer un clone primero (no hay have lines)
    let refs_recibidas = comunicacion.obtener_lineas().unwrap();
    let capacidades = refs_recibidas[0].split("\0").collect::<Vec<&str>>()[1];
    let wants = comunicacion.obtener_wants(&refs_recibidas, capacidades.to_string()).unwrap();
    comunicacion.responder(wants.clone()).unwrap();
    comunicacion.responder(vec![io::obtener_linea_con_largo_hex("done")]).unwrap();
    let nak = comunicacion.obtener_lineas().unwrap();
    println!("nak: {:?}", nak);

    println!("Obteniendo paquete..");
    let mut packfile = comunicacion.obtener_lineas_como_bytes().unwrap();
    comunicacion.obtener_paquete(&mut packfile).unwrap();

    Ok(())
}

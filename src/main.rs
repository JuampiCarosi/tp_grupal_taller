use std::net::TcpStream;
use std::io::{Write, Read};
use gir::{io, comunicacion, packfile};
fn main() -> std::io::Result<()> {
    // Configura la dirección del servidor y el puerto
    // let server_address = "127.0.0.1:9418"; // Cambia la dirección IP si es necesario

    // // Conéctate al servidor
    // let mut client = TcpStream::connect(server_address)?;

    // // Envía datos al servidor (reemplaza esto por tus datos Git)
    // let request_data = "git-upload-pack /.git/\0host=example.com\0\0version=1\0";
    // let largo_hex = io::calcular_largo_hex(request_data);
    // let a = format!("{}{}", largo_hex, request_data);
    // client.write_all(a.as_bytes())?;

    // // aca depende del comando, pero voy a hacer un clone primero (no hay have lines)
    // let refs_recibidas = comunicacion::obtener_lineas(&mut client).unwrap();
    // // println!("refs recibidas: {:?}", refs_recibidas);
    // let capacidades = refs_recibidas[0].split("\0").collect::<Vec<&str>>()[1];
    // let wants = comunicacion::obtener_wants(&refs_recibidas, capacidades.to_string()).unwrap();
    // comunicacion::responder(&mut client, wants).unwrap();

    Ok(())

}

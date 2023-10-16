use std::net::TcpStream;
use std::io::{Write, Read};
use taller::comunicacion::Comunicacion;
fn main() -> std::io::Result<()> {
    // Configura la dirección del servidor y el puerto
    let server_address = "127.0.0.1:9418"; // Cambia la dirección IP si es necesario

    // Conéctate al servidor
    let mut client = TcpStream::connect(server_address)?;

    // Envía datos al servidor (reemplaza esto por tus datos Git)
    let request_data = "git-upload-pack /.git/\0host=example.com\0\0version=1\0";
    let largo_hex = Comunicacion::calcular_largo_hex(request_data);
    let a = format!("{}{}", largo_hex, request_data);
    client.write_all(a.as_bytes())?;

    // Recibe la respuesta del servidor (ajusta el tamaño del búfer según tus necesidades)
    // let mut response = [0; 4096];
    // let bytes_read = client.read(&mut response)?;

    // Procesa la respuesta del servidor aquí

    Ok(())
}
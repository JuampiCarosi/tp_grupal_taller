use std::net::TcpStream;
use std::io::{Write, Read};

fn main() -> std::io::Result<()> {
    // Configura la dirección del servidor y el puerto
    let server_address = "127.0.0.1:9418"; // Cambia la dirección IP si es necesario

    // Conéctate al servidor
    let mut client = TcpStream::connect(server_address)?;

    // Envía datos al servidor (reemplaza esto por tus datos Git)
    let request_data = b"0045git-upload-pack /schacon/gitbook.git\0host=example.com\0\0version=1\0";
    client.write_all(request_data)?;

    // Recibe la respuesta del servidor (ajusta el tamaño del búfer según tus necesidades)
    // let mut response = [0; 4096];
    // let bytes_read = client.read(&mut response)?;

    // Procesa la respuesta del servidor aquí

    Ok(())
}
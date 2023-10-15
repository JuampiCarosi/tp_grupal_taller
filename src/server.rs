use std::env::args;
use std::io::{BufRead, BufReader, Read, Write, Error};
use std::net::{TcpListener, TcpStream};
static SERVER_ARGS: usize = 2;

fn main() -> Result<(), ()> {
    let argv = args().collect::<Vec<String>>();
    if argv.len() != SERVER_ARGS {
        println!("Cantidad de argumentos inválido");
        let app_name = &argv[0];
        println!("Usage:\n{:?} <puerto>", app_name);
        return Err(());
    }

    let address = "127.0.0.1:".to_owned() + &argv[1];
    server_run(&address).unwrap();
    Ok(())
}

fn server_run(address: &str) -> std::io::Result<()> {
    let listener = TcpListener::bind(address)?;
    // accept devuelve una tupla (TcpStream, std::net::SocketAddr)
    let (mut client_stream, socket_addr) = listener.accept()?;
    println!("La socket addr del client: {:?}", socket_addr);
    // let mut client_stream : TcpStream = connection.0;
    // TcpStream implementa el trait Read, así que podemos trabajar como si fuera un archivo
    handle_client(&mut client_stream)?;
    Ok(())
}

// 
// fn handle_client(stream: &mut dyn Read) -> Result<(), Error> {
    

    // let mut bytes_to_read = [0; 4];
    
    // let length = u32::from_str_radix(std::str::from_utf8(&length_str)?, 10)?;
    
    // let mut data = vec![0; length as usize];
    // stream.read_exact(&mut data)?;

    // let reader = BufReader::new(stream);
    // let mut lines = reader.lines();
    // // iteramos las lineas que recibimos de nuestro cliente
    // while let Some(Ok(line)) = lines.next() {
    //     println!("Recibido: {:?}", line);
    // }
//     Ok(())
// }
fn handle_client(stream: &mut TcpStream) -> std::io::Result<()> {
    // lee primera parte, 4 bytes en hexadecimal indican el largo del stream
    let mut length_bytes = [0; 4];
    stream.read_exact(&mut length_bytes)?;
    // largo de bytes a str
    let length_str = std::str::from_utf8(&length_bytes).unwrap(); 
    // transforma str a u32
    let length = u32::from_str_radix(length_str, 16).unwrap();
    println!("length: {:?}", length);

    // lee el resto del stream
    let mut data = vec![0; length as usize];
    stream.read(&mut data)?;
    let line = String::from_utf8(data).unwrap();
    println!("Received: {:?}", line);
    println!("length: {:?}", calcular_largo(&line));

    // stream.write(line.as_bytes())?;

    Ok(())
}


fn calcular_largo(line: &str) -> u32 {
    let largo = line.len();
    let largo_hex = format!("{:x}", largo);
    let largo_hex = largo_hex.as_bytes();
    let mut largo_bytes = [0; 4];
    largo_bytes.copy_from_slice(&largo_hex);
    u32::from_be_bytes(largo_bytes)
}
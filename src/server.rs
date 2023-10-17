use std::env::{self, args};
use std::io::{BufRead, BufReader, Read, Write, Error};
use std::net::{TcpListener, TcpStream};
use crate::comunicacion::Comunicacion;
use crate::err_comunicacion::ErrorDeComunicacion;


pub struct Servidor { 
    com: Comunicacion,
    dir: String 
}

impl Servidor { 

    pub fn new(address: &str) -> Servidor {
        let listener = TcpListener::bind(address).unwrap();
        let dir = env!("CARGO_MANIFEST_DIR").to_string();

        let com = Comunicacion::new(listener, dir.clone());
        Servidor { com, dir }
    }

    pub fn server_run(&mut self) -> Result<(),ErrorDeComunicacion> {
        // loop {
        //     self.com.procesar_datos()?;
        // }
        self.com.procesar_datos()?; 
        Ok(())
    }

//     fn handle_client(stream: &mut TcpStream) -> std::io::Result<()> {
//         // lee primera parte, 4 bytes en hexadecimal indican el largo del stream
//         let mut length_bytes = [0; 4];
//         stream.read_exact(&mut length_bytes)?;
//         // largo de bytes a str
//         let length_str = std::str::from_utf8(&length_bytes).unwrap(); 
//         // transforma str a u32
//         let length = u32::from_str_radix(length_str, 16).unwrap();
//         println!("length: {:?}", length);
    
//         // lee el resto del stream
//         let mut data = vec![0; (length - 4) as usize];
//         stream.read_exact(&mut data)?;
//         let line = String::from_utf8(data).unwrap();
//         println!("Received: {:?}", line);
//         println!("length: {:?}", Self::calcular_largo_hex(&line));
//         let mut action = line.split_whitespace();        
//         println!("action: {:?}", action.next().unwrap());
//         println!("action: {:?}", action.next().unwrap());
//         // stream.write(line.as_bytes())?;
    
//         Ok(())
//     }

    
// fn calcular_largo_hex(line: &str) -> String {
//     let largo = line.len() + 4; // el + 4 es por los 4 bytes que indican el largo
//     let largo_hex = format!("{:x}", largo);
//     format!("{:0>4}", largo_hex)
// }

}

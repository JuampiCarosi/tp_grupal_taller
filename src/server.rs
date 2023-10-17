use std::net::{TcpListener, TcpStream};
use std::io::{self, Write, Read, BufRead};
use std::env;
use std::path::PathBuf;
use std::str;
use crate::err_comunicacion::ErrorDeComunicacion;
use crate::io as git_io;
pub struct Servidor { 
    listener: TcpListener,
    dir: String,
    capacidades: Vec<String>,
}

impl Servidor { 

    pub fn new(address: &str) -> std::io::Result<Servidor> {
        let listener = TcpListener::bind(address)?;
        let dir = env!("CARGO_MANIFEST_DIR").to_string();
        let capacidades: Vec<String> = vec!["multi_ack", "thin-pack", "side-band", "side-band-64k", "ofs-delta", "shallow", "no-progress", "include-tag"].iter().map(|x| x.to_string()).collect();
        Ok(Servidor { listener, dir, capacidades })
    }

    pub fn server_run(&mut self) -> Result<(),ErrorDeComunicacion> {
        // loop {
        //     self.com.procesar_datos()?;
        // }
        let (mut stream, _) = self.listener.accept()?;
        self.manejar_cliente(&mut stream)?; 
        Ok(())
    }

    pub fn manejar_cliente(&mut self, stream: &mut TcpStream) -> Result<(), ErrorDeComunicacion> {
        // lee primera parte, 4 bytes en hexadecimal indican el largo del stream
        let mut tamanio_bytes = [0; 4];
        stream.read_exact(&mut tamanio_bytes)?;
        // largo de bytes a str
        let tamanio_str = str::from_utf8(&tamanio_bytes)?;
        // transforma str a u32
        let tamanio = u32::from_str_radix(tamanio_str, 16).unwrap();

        // lee el resto del stream
        let mut data = vec![0; (tamanio - 4) as usize];
        stream.read_exact(&mut data)?;
        let line = str::from_utf8(&data)?;
        println!("Received: {:?}", line);
        self.parse_line(&line);
    //     // stream.write(line.as_bytes())?;
        Ok(())
    }


    fn parse_line(&mut self, line: &str) -> Result<(), ErrorDeComunicacion>{
        let req: Vec<&str> = line.split_whitespace().collect();
        // veo si es un comando git
        match req[0] {
            "git-upload-pack" => {
                println!("git-upload-pack");
                let args: Vec<_> = req[1].split('\0').collect();    
                let path = PathBuf::from(self.dir.clone() + args[0]);
                let mut refs: Vec<String> = Vec::new();
                refs.append(&mut git_io::obtener_refs(&mut path.join("HEAD"))?);
                refs.append(&mut git_io::obtener_refs(&mut path.join("refs/heads/"))?);
                refs.append(&mut git_io::obtener_refs(&mut path.join("refs/tags/"))?);
                refs[0] = self.agregar_capacidades(refs[0].clone());
                println!("refs: {:?}", refs);
                Ok(())
            },
            _ =>    {
                println!("No se reconoce el comando");
                // cambiar el error
                Err(ErrorDeComunicacion::IoError(io::Error::new(io::ErrorKind::NotFound, "No existe el comando")))
            },
        }
    }

    fn agregar_capacidades(&self, referencia: String) -> String {
        let mut referencia_con_capacidades = referencia + "\0"; 
        for cap in self.capacidades.iter() {
            referencia_con_capacidades.push_str(&format!("{} ", cap));
        }
        git_io::obtener_linea_con_largo_hex(&referencia_con_capacidades)
    }

}

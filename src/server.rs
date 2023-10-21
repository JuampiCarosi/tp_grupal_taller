use std::net::{TcpListener, TcpStream};
use std::io::{self, Write, Read, BufRead};
use std::env;
use std::path::PathBuf;
use std::str;
use crate::err_comunicacion::ErrorDeComunicacion;
use crate::{io as git_io, comunicacion};
pub struct Servidor { 
    listener: TcpListener,
    dir: String,
    capacidades: Vec<String>,
}

impl Servidor { 

    pub fn new(address: &str) -> std::io::Result<Servidor> {
        let listener = TcpListener::bind(address)?;
        // busca la carpeta raiz del proyecto (evita hardcodear la ruta)
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
        let pedido = comunicacion::aceptar_pedido(stream)?; // acepto la primera linea
        let respuesta = self.parse_line(&pedido)?; // parse de la liena para ver que se pide
        comunicacion::responder(stream, respuesta)?; // respondo con las refs (en caso de que sea upload-pack)
        
        let wants = comunicacion::obtener_lineas(stream)?; // obtengo los wants del cliente 
        
        // a partir de aca se asume que va a ser un clone porque es el caso mas sencillo, despues cambiar
        if wants.ends_with(&["0009done\n".to_string()]) {
            let mut wants = comunicacion::obtener_obj_ids(&wants);
            comunicacion::responder(stream, vec![git_io::obtener_linea_con_largo_hex("NAK\n")])? // respondo NAK
        }

        Ok(())
    }


    fn parse_line(&mut self, line: &str) -> Result<Vec<String>, ErrorDeComunicacion>{
        let req: Vec<&str> = line.split_whitespace().collect();
        // veo si es un comando git
        match req[0] {
            "git-upload-pack" => {
                println!("git-upload-pack");
                let args: Vec<_> = req[1].split('\0').collect();    
                let path = PathBuf::from(self.dir.clone() + args[0]);
                let mut refs: Vec<String> = Vec::new();
                if let Ok(mut head) = git_io::obtener_refs(&mut path.join("HEAD")) {
                    refs.append(&mut head);
                }
                refs.append(&mut git_io::obtener_refs(&mut path.join("refs/heads/"))?);
                refs.append(&mut git_io::obtener_refs(&mut path.join("refs/tags/"))?);
                refs[0] = self.agregar_capacidades(refs[0].clone());
                // println!("refs: {:?}", refs);
                Ok(refs)
            },
            _ =>    {
                println!("No se reconoce el comando");
                // cambiar el error
                Err(ErrorDeComunicacion::IoError(io::Error::new(io::ErrorKind::NotFound, "No existe el comando")))
            },
        }
    }

    fn agregar_capacidades(&self, referencia: String) -> String {
        let mut referencia_con_capacidades = referencia.split_at(4).1.to_string() + "\0";  // borro los primeros 4 caracteres que quedan del tamanio anterior
        for cap in self.capacidades.iter() {
            referencia_con_capacidades.push_str(&format!("{} ", cap));
        }
        let mut referencia_con_capacidades = referencia_con_capacidades.trim_end().to_string();
        referencia_con_capacidades.push_str("\n");
        git_io::obtener_linea_con_largo_hex(&referencia_con_capacidades)
    }

}

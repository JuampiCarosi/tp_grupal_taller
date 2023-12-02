use crate::servidor::{upload_pack::upload_pack, receive_pack::receive_pack};
// use crate::utils::server_utils;
use crate::tipos_de_dato::{comunicacion::Comunicacion, comunicacion::RespuestaDePedido, logger::Logger};
use crate::utils::io as gir_io;
use std::{env, net::{TcpListener, TcpStream}, path::PathBuf, str, sync::Arc, thread};

const VERSION: &str = "version 1";
const CAPABILITIES: &str = "ofs-delta symref=HEAD:refs/heads/master agent=git/2.17.1";
const DIR: &str = "/srv"; // direccion relativa

pub struct Servidor {
    listener: TcpListener,
    threads: Vec<Option<thread::JoinHandle<Result<(), String>>>>,
    logger: Arc<Logger>,
}

impl Servidor {
    pub fn new(address: &str, logger: Arc<Logger>) -> std::io::Result<Servidor> {
        let listener = TcpListener::bind(address)?;
        println!("Escuchando en {}", address);

        Ok(Servidor { listener, threads: Vec::new(), logger })
    }

    pub fn iniciar_servidor(&mut self) -> Result<(), String> {
        while let Ok((stream, socket)) = self.listener.accept() {
            println!("Conectado al cliente {:?}", socket);
            let logge_clone = self.logger.clone();

            let handle = thread::spawn(move || -> Result<(), String>{
                let mut comunicacion =
                    Comunicacion::new_para_testing(stream.try_clone().unwrap(), logge_clone);
                Self::manejar_cliente(
                    &mut comunicacion,
                    &(env!("CARGO_MANIFEST_DIR").to_string() + DIR),
                )?;
                Ok(())
            });
            self.threads.push(Some(handle));
        }
        Ok(())
    }

    pub fn manejar_cliente(
        comunicacion: &mut Comunicacion<TcpStream>,
        dir: &str,
    ) -> Result<(), String> {
        loop {
            let pedido = match comunicacion.aceptar_pedido()? {
                RespuestaDePedido::Mensaje(mensaje) => mensaje,
                RespuestaDePedido::Terminate => break,
            }; // acepto la primera linea
            Self::procesar_pedido(&pedido, comunicacion, dir)?; // parse de la liena para ver que se pide
        }
        Ok(())
    }


    fn parsear_linea_pedido_y_responder_con_version(linea_pedido: &str, comunicacion: &mut Comunicacion<TcpStream>, dir: &str) -> Result<(String, String), String> {

        let pedido: Vec<String> = linea_pedido.split_whitespace().into_iter().map(|s| s.to_string()).collect();
        let args: Vec<String> = pedido[1].split('\0').into_iter().map(|s| s.to_string()).collect();
        let dir_repo = dir.to_string() + &args[0];
        comunicacion
            .enviar_linea(&gir_io::obtener_linea_con_largo_hex(
                &(VERSION.to_string() + "\n"),
            ))
            .unwrap();
        let pedido = &pedido[0];
        Ok((pedido.to_owned(), dir_repo))
    }
    
    fn procesar_pedido(
        linea: &str,
        comunicacion: &mut Comunicacion<TcpStream>,
        dir: &str,
    ) -> Result<(), String> {
        let (pedido, dir_repo) = Self::parsear_linea_pedido_y_responder_con_version(linea, comunicacion, dir)?;

        let refs: Vec<String>;

        match pedido.as_str() {
            "git-upload-pack" => {
                println!("upload-pack recibido, ejecutando");
                refs = server_utils::obtener_refs_de(PathBuf::from(&dir_repo));
                comunicacion.responder(refs).unwrap();
                upload_pack(dir_repo, comunicacion)
            }
            "git-receive-pack" => {
                println!("receive-pack recibido, ejecutando");
                let path = PathBuf::from(&dir_repo);

                if !path.exists() {
                    gir_io::crear_directorio(&path.join("refs/")).unwrap();
                    gir_io::crear_directorio(&path.join("refs/heads/")).unwrap();
                    gir_io::crear_directorio(&path.join("refs/tags/")).unwrap();
                }
                refs = server_utils::obtener_refs_de(path);
                if refs.is_empty() {
                    comunicacion
                        .responder(vec![server_utils::agregar_capacidades("0".repeat(40))])
                        .unwrap();
                } else {
                    comunicacion.responder(refs).unwrap();
                }
                receive_pack(dir_repo.to_string(), comunicacion)
            }
            _ => Err("No existe el comando".to_string())
        }
    }

    
}


impl Drop for Servidor {
    fn drop(&mut self) {
        for thread in self.threads.drain(..) {
            if let Some(thread) = thread {
                if let Err(e) = thread.join() {
                    println!("Error en el thread: {:?}", e);
                }
            }
        }
        println!("Servidor cerrado");
    }
}

// -------------- utils del server -------------- 
mod server_utils { 
    use super::*;
    pub fn obtener_refs_de(dir: PathBuf) -> Vec<String> {
        let mut refs: Vec<String> = Vec::new();
        let head_ref = gir_io::obtener_ref_head(dir.join("HEAD"));
        if let Ok(head) = head_ref {
            refs.push(head)
        }
        gir_io::obtener_refs_con_largo_hex(
            &mut refs,
            dir.join("refs/heads/"),
            dir.to_str().unwrap(),
        )
        .unwrap();
        gir_io::obtener_refs_con_largo_hex(
            &mut refs,
            dir.join("refs/tags/"),
            dir.to_str().unwrap(),
        )
        .unwrap();
        if !refs.is_empty() {
            let ref_con_cap = agregar_capacidades(refs[0].clone());
            refs.remove(0);
            refs.insert(0, ref_con_cap);
        }
        refs
    }



    pub fn agregar_capacidades(referencia: String) -> String {
        let mut referencia_con_capacidades: String;
        if referencia.len() > 40 {
            referencia_con_capacidades = referencia.split_at(4).1.to_string() + "\0";
        // borro los primeros 4 caracteres que quedan del tamanio anterior
        } else {
            referencia_con_capacidades = referencia + "\0";
        }
        let capacidades: Vec<&str> = CAPABILITIES.split_whitespace().collect();
        for cap in capacidades.iter() {
            referencia_con_capacidades.push_str(&format!("{} ", cap));
        }
        let mut referencia_con_capacidades = referencia_con_capacidades.trim_end().to_string();
        referencia_con_capacidades.push('\n');
        gir_io::obtener_linea_con_largo_hex(&referencia_con_capacidades)
    }
}


#[cfg(test)]
mod tests { 
    use super::*;
    
    #[test]
    fn test01() { 

    }
}
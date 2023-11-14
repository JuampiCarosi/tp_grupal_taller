use crate::err_comunicacion::ErrorDeComunicacion;
use crate::servidor::receive_pack::receive_pack;
use crate::servidor::upload_pack::upload_pack;
use crate::tipos_de_dato::comunicacion::Comunicacion;
use crate::tipos_de_dato::comunicacion::RespuestaDePedido;
use crate::tipos_de_dato::logger::Logger;
use crate::utils::io as gir_io;
use std::env;
use std::io;
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::str;
use std::sync::Arc;
use std::thread;

const VERSION: &str = "version 1";
const CAPABILITIES: &str = "multi_ack thin-pack side-band side-band-64k ofs-delta shallow no-progress include-tag multi_ack_detailed no-done symref=HEAD:refs/heads/master agent=git/2.17.1";
const DIR: &str = "/srv"; // direccion relativa

pub struct Servidor {
    listener: TcpListener,
    logger: Arc<Logger>,
}

impl Servidor {
    pub fn new(address: &str, logger: Arc<Logger>) -> std::io::Result<Servidor> {
        let listener = TcpListener::bind(address)?;
        println!("Escuchando en {}", address);

        Ok(Servidor { listener, logger })
    }

    pub fn iniciar_servidor(&self) -> Result<(), ErrorDeComunicacion> {
        while let Ok((stream, socket)) = self.listener.accept() {
            println!("Conectado al cliente {:?}", socket);
            let logge_clone = self.logger.clone();
            thread::spawn(move || {
                let mut comunicacion = Comunicacion::new(stream.try_clone().unwrap(), logge_clone);
                Self::manejar_cliente(
                    &mut comunicacion,
                    &(env!("CARGO_MANIFEST_DIR").to_string() + DIR),
                )
                .unwrap();
            });
        }
        Ok(())
    }

    pub fn manejar_cliente(
        comunicacion: &mut Comunicacion<TcpStream>,
        dir: &str,
    ) -> Result<(), ErrorDeComunicacion> {
        loop {
            let pedido = match comunicacion.aceptar_pedido()? {
                RespuestaDePedido::Mensaje(mensaje) => mensaje,
                RespuestaDePedido::Terminate => break,
            }; // acepto la primera linea
            Self::parse_line(&pedido, comunicacion, dir)?; // parse de la liena para ver que se pide
        }
        Ok(())
    }

    fn parse_line(
        linea: &str,
        comunicacion: &mut Comunicacion<TcpStream>,
        dir: &str,
    ) -> Result<(), ErrorDeComunicacion> {
        let pedido: Vec<&str> = linea.split_whitespace().collect();
        let args: Vec<_> = pedido[1].split('\0').collect();
        let dir_repo = dir.to_string() + args[0];
        let refs: Vec<String>;
        comunicacion
            .enviar_linea(gir_io::obtener_linea_con_largo_hex(
                &(VERSION.to_string() + "\n"),
            ))
            .unwrap();

        match pedido[0] {
            "git-upload-pack" => {
                println!("upload-pack recibido, ejecutando");
                refs = Self::obtener_refs_de(PathBuf::from(&dir_repo));
                comunicacion.responder(refs).unwrap();
                upload_pack(dir_repo, comunicacion)
            }
            "git-receive-pack" => {
                println!("receive-pack recibido, ejecutando");
                let path = PathBuf::from(dir_repo);

                if !path.exists() {
                    gir_io::crear_directorio(&path.join("refs/")).unwrap();
                    gir_io::crear_directorio(&path.join("refs/heads/")).unwrap();
                    gir_io::crear_directorio(&path.join("refs/tags/")).unwrap();
                }
                refs = Self::obtener_refs_de(path);
                if refs.is_empty() {
                    comunicacion
                        .responder(vec![Self::agregar_capacidades("0".repeat(40))])
                        .unwrap();
                } else {
                    comunicacion.responder(refs).unwrap();
                }
                receive_pack(dir.to_string(), comunicacion)
            }
            _ => Err(ErrorDeComunicacion::IoError(io::Error::new(
                io::ErrorKind::NotFound,
                "No existe el comando",
            ))),
        }
    }

    // esto esta hardcodeado, hay que cambiarlo
    // devuelve las refs de un directorio valido
    fn obtener_refs_de(dir: PathBuf) -> Vec<String> {
        let mut refs: Vec<String> = Vec::new();
        let head_ref = gir_io::obtener_ref_head(dir.join("HEAD"));
        match head_ref {
            Ok(head) => refs.push(head),
            Err(_) => {}
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
            let ref_con_cap = Self::agregar_capacidades(refs[0].clone());
            refs.remove(0);
            refs.insert(0, ref_con_cap);
        }
        refs
    }

    fn agregar_capacidades(referencia: String) -> String {
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

// pub fn server_run(&self) -> Result<(), ErrorDeComunicacion> {
//     let ref_self = Arc::new(self);
//     while let Ok((stream, _)) = self.listener.accept() {
//         let mut comunicacion = Comunicacion::new(stream.try_clone().unwrap());
//         let ref_self = self.clone(); // Clonar el servidor

//         thread::spawn(move || {
//             // Dentro de la clausura, se captura self_clone en lugar de self.
//             ref_self.manejar_cliente(&mut comunicacion).unwrap_or_else(|error| eprintln!("{:?}", error));
//         });
//     }
//     Ok(())
// }

// pub fn manejar_cliente(
//     &self,
//     comunicacion: &mut Comunicacion,
// ) -> Result<(), ErrorDeComunicacion> {
//     let pedido = comunicacion.aceptar_pedido()?; // acepto la primera linea
//     self.parse_line(&pedido, comunicacion)?; // parse de la liena para ver que se pide

//     Ok(())
// }

// fn parse_line(&self, linea: &str, comunicacion: &mut Comunicacion) -> Result<(), ErrorDeComunicacion> {
//     let pedido: Vec<&str> = linea.split_whitespace().collect();
//     // veo si es un comando git
//     let args: Vec<_> = pedido[1].split('\0').collect();
//     let path = PathBuf::from(self.dir.clone() + args[0]);
//     let refs: Vec<String>;
//     match pedido[0] {
//         "git-upload-pack" => {
//             refs = self.obtener_refs_de(path);
//             comunicacion.responder(refs).unwrap();
//             upload_pack(self.dir.clone(), comunicacion)
//         },
//         "git-receive-pack" => {
//             if !path.exists() {
//                 crear_directorio(&path.join("refs/")).unwrap();
//                 crear_directorio(&path.join("refs/heads/")).unwrap();
//                 crear_directorio(&path.join("refs/tags/")).unwrap();
//             }
//             refs = self.obtener_refs_de(path);
//             comunicacion.responder(refs).unwrap();
//             receive_pack(self.dir.clone(), comunicacion)
//             // Ok(())
//         },
//         _ => {
//             println!("No se reconoce el comando");
//             // cambiar el error
//             return Err(ErrorDeComunicacion::IoError(io::Error::new(
//                 io::ErrorKind::NotFound,
//                 "No existe el comando",
//             )));
//         }
//     }
// }
// // devuelve las refs de un directorio valido
// fn obtener_refs_de(&self, dir: PathBuf) -> Vec<String> {
//     // println!("path del comando: {:?}", dir);
//     let mut refs: Vec<String> = Vec::new();
//     if let Ok(mut head) = gir_io::obtener_refs_con_largo_hex(dir.join("HEAD"), "/home/juani/23C2-Cangrejos-Tacticos/srv/.gir/".to_string()) {
//         refs.append(&mut head);
//     }
//     println!("Hola!");
//     refs.append(&mut gir_io::obtener_refs_con_largo_hex(dir.join("refs/heads/"), String::from("/home/juani/23C2-Cangrejos-Tacticos/srv/.gir/")).unwrap());
//     refs.append(&mut gir_io::obtener_refs_con_largo_hex(dir.join("refs/tags/"), String::from("/home/juani/23C2-Cangrejos-Tacticos/srv/.gir/")).unwrap());
//     if !refs.is_empty(){
//         refs.insert(0, self.agregar_capacidades(refs[0].clone()));
//     }
//     refs
// }

// fn agregar_capacidades(&self, referencia: String) -> String {
//     let mut referencia_con_capacidades = referencia.split_at(4).1.to_string() + "\0"; // borro los primeros 4 caracteres que quedan del tamanio anterior
//     for cap in self.capacidades.iter() {
//         referencia_con_capacidades.push_str(&format!("{} ", cap));
//     }
//     let mut referencia_con_capacidades = referencia_con_capacidades.trim_end().to_string();
//     referencia_con_capacidades.push('\n');
//     gir_io::obtener_linea_con_largo_hex(&referencia_con_capacidades)
// }
// }

// git daemon [--verbose] [--syslog] [--export-all]
// [--timeout=<n>] [--init-timeout=<n>] [--max-connections=<n>]
// [--strict-paths] [--base-path=<path>] [--base-path-relaxed]
// [--user-path | --user-path=<path>]
// [--interpolated-path=<pathtemplate>]
// [--reuseaddr] [--detach] [--pid-file=<file>]
// [--enable=<service>] [--disable=<service>]
// [--allow-override=<service>] [--forbid-override=<service>]
// [--access-hook=<path>] [--[no-]informative-errors]
// [--inetd |
//  [--listen=<host_or_ipaddr>] [--port=<n>]
//  [--user=<user> [--group=<group>]]]
// [--log-destination=(stderr|syslog|none)]
// [<directory>...]

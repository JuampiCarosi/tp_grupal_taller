use crate::err_comunicacion::ErrorDeComunicacion;
use crate::io::crear_directorio;
use crate::receive_pack::receive_pack;
use crate::upload_pack::upload_pack;
use crate::{comunicacion::Comunicacion, io as git_io};
use std::env;
use std::io;
use std::net::TcpListener;
use std::path::PathBuf;
use std::str;
pub struct Servidor {
    listener: TcpListener,
    dir: String,
    capacidades: Vec<String>,
}

impl Servidor {
    pub fn new(address: &str) -> std::io::Result<Servidor> {
        let listener = TcpListener::bind(address)?;
        // busca la carpeta raiz del proyecto (evita hardcodear la ruta)
        let dir = env!("CARGO_MANIFEST_DIR").to_string() + "/srv";
        // esto es para checkear, no tengo implementado nada de lo que dice xd
        let capacidades: Vec<String> = [
            "multi_ack",
            "thin-pack",
            "side-band",
            "side-band-64k",
            "ofs-delta",
            "shallow",
            "no-progress",
            "include-tag",
        ]
        .iter()
        .map(|x| x.to_string())
        .collect();
        Ok(Servidor {
            listener,
            dir,
            capacidades,
        })
    }

    pub fn server_run(&mut self) -> Result<(), ErrorDeComunicacion> {
        // loop {
        //     self.com.procesar_datos()?;
        // }
        let (stream, _) = self.listener.accept()?;
        self.manejar_cliente(&mut Comunicacion::new(stream))?;
        Ok(())
    }

    pub fn manejar_cliente(
        &mut self,
        comunicacion: &mut Comunicacion,
    ) -> Result<(), ErrorDeComunicacion> {
        let pedido = comunicacion.aceptar_pedido()?; // acepto la primera linea
        self.parse_line(&pedido, comunicacion)?; // parse de la liena para ver que se pide

        Ok(())
    }

    fn parse_line(
        &mut self,
        linea: &str,
        comunicacion: &mut Comunicacion,
    ) -> Result<(), ErrorDeComunicacion> {
        let pedido: Vec<&str> = linea.split_whitespace().collect();
        // veo si es un comando git
        let args: Vec<_> = pedido[1].split('\0').collect();
        let path = PathBuf::from(self.dir.clone() + args[0]);
        let refs: Vec<String>;
        match pedido[0] {
            "git-upload-pack" => {
                refs = self.obtener_refs_de(path);
                comunicacion.responder(refs).unwrap();
                upload_pack(self.dir.clone(), comunicacion)
            }
            "git-receive-pack" => {
                if !path.exists() {
                    crear_directorio(&path.join("refs/")).unwrap();
                    crear_directorio(&path.join("refs/heads/")).unwrap();
                    crear_directorio(&path.join("refs/tags/")).unwrap();
                }
                refs = self.obtener_refs_de(path);
                comunicacion.responder(refs).unwrap();
                receive_pack(self.dir.clone(), comunicacion)
                // Ok(())
            }
            _ => {
                println!("No se reconoce el comando");
                // cambiar el error
                Err(ErrorDeComunicacion::IoError(io::Error::new(
                    io::ErrorKind::NotFound,
                    "No existe el comando",
                )))
            }
        }
    }
    // devuelve las refs de un directorio valido
    fn obtener_refs_de(&self, dir: PathBuf) -> Vec<String> {
        // println!("path del comando: {:?}", dir);
        let mut refs: Vec<String> = Vec::new();
        if let Ok(mut head) = git_io::obtener_refs_con_largo_hex(
            dir.join("HEAD"),
            "/home/juani/23C2-Cangrejos-Tacticos/srv/.gir/".to_string(),
        ) {
            refs.append(&mut head);
        }
        println!("Hola!");
        refs.append(
            &mut git_io::obtener_refs_con_largo_hex(
                dir.join("refs/heads/"),
                String::from("/home/juani/23C2-Cangrejos-Tacticos/srv/.gir/"),
            )
            .unwrap(),
        );
        refs.append(
            &mut git_io::obtener_refs_con_largo_hex(
                dir.join("refs/tags/"),
                String::from("/home/juani/23C2-Cangrejos-Tacticos/srv/.gir/"),
            )
            .unwrap(),
        );
        if !refs.is_empty() {
            refs.insert(0, self.agregar_capacidades(refs[0].clone()));
        }
        refs
    }

    fn agregar_capacidades(&self, referencia: String) -> String {
        let mut referencia_con_capacidades = referencia.split_at(4).1.to_string() + "\0"; // borro los primeros 4 caracteres que quedan del tamanio anterior
        for cap in self.capacidades.iter() {
            referencia_con_capacidades.push_str(&format!("{} ", cap));
        }
        let mut referencia_con_capacidades = referencia_con_capacidades.trim_end().to_string();
        referencia_con_capacidades.push('\n');
        git_io::obtener_linea_con_largo_hex(&referencia_con_capacidades)
    }
}

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

use crate::err_comunicacion::ErrorDeComunicacion;
use crate::io::obtener_archivos_faltantes;
use crate::packfile;
use crate::{comunicacion::Comunicacion, io as git_io};
use std::env;
use std::io;
use std::net::TcpListener;
use std::path::PathBuf;
use std::str;
use crate::utilidades_strings;
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
        let respuesta = self.parse_line(&pedido)?; // parse de la liena para ver que se pide
        comunicacion.responder(respuesta)?; // respondo con las refs 
        
        // caso push 
        // let refs_a_actualizar = comunicacion.obtener_lineas().unwrap();
        

        let mut wants = comunicacion.obtener_lineas()?; // obtengo los wants del cliente

        // ------- CLONE --------
        // a partir de aca se asume que va a ser un clone porque es el caso mas sencillo, despues cambiar
        let mut lineas_siguientes = comunicacion.obtener_lineas()?;
        if lineas_siguientes[0].clone().contains("done") {
            let want_obj_ids = utilidades_strings::eliminar_prefijos(&mut wants, "want");
            comunicacion.responder(vec![git_io::obtener_linea_con_largo_hex("NAK\n")])?; // respondo NAK
            println!("want_obj_ids: {:?}", want_obj_ids);
            let packfile =
                packfile::Packfile::new().obtener_pack_entero(self.dir.clone() + "/.git/objects/"); // obtengo el packfile
                // git_io::leer_bytes("./.git/objects/pack/pack-31897a1f902980a7e540e812b54f5702f449af8b.pack").unwrap();
            comunicacion.responder_con_bytes(packfile).unwrap();
            return Ok(());
        }

        // -------- pull / fetch ----------
        let have_obj_ids = utilidades_strings::eliminar_prefijos(&mut lineas_siguientes, "have");
        let respuesta_acks_nak = git_io::obtener_ack(have_obj_ids.clone(), self.dir.clone() + "/.git/objects/");
        comunicacion.responder(respuesta_acks_nak).unwrap();

        let faltantes = obtener_archivos_faltantes(have_obj_ids, self.dir.clone());
        // obtener un packfile de los faltantes...
        let packfile = packfile::Packfile::new().obtener_pack_con_archivos(faltantes);
        comunicacion.responder_con_bytes(packfile).unwrap();
        // enviar
        Ok(())
    }

    fn parse_line(&mut self, line: &str) -> Result<Vec<String>, ErrorDeComunicacion> {
        let pedido: Vec<&str> = line.split_whitespace().collect();
        // veo si es un comando git
        match pedido[0] {
            "git-upload-pack" => {
                println!("git-upload-pack");
                let args: Vec<_> = pedido[1].split('\0').collect();
                let path = PathBuf::from(self.dir.clone() + args[0]);
                let mut refs: Vec<String> = Vec::new();
                if let Ok(mut head) = git_io::obtener_refs(&mut path.join("HEAD")) {
                    refs.append(&mut head);
                }
                refs.append(&mut git_io::obtener_refs(&mut path.join("refs/heads/"))?);
                refs.append(&mut git_io::obtener_refs(&mut path.join("refs/tags/"))?);
                refs[0] = self.agregar_capacidades(refs[0].clone());
                Ok(refs)
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

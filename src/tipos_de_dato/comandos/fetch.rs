use crate::io::escribir_bytes;
use crate::utilidades_path_buf;
use crate::{
    comunicacion::Comunicacion, io, packfile, tipos_de_dato::comandos::write_tree,
    tipos_de_dato::objetos::tree::Tree,
};
use std::io::Write;
use std::net::TcpStream;
use std::path::PathBuf;
use std::rc::Rc;

use super::remote;

pub struct Fetch {
    remoto: String,
}

impl Fetch {
    pub fn new() -> Self {
        let remoto = "origin".to_string();
        //"Por ahora lo hardcoedo necesito el config que no esta en esta rama".to_string();
        Fetch { remoto }
    }

    //verificar si existe /.git
    pub fn ejecutar(&mut self) -> Result<String, String> {
        let direccion_servidor = "127.0.0.1:9418"; // Cambia la dirección IP si es necesario
                                                   //se inicia la comunicacon con servidor
        let mut comunicacion = Comunicacion::new_desde_direccion_servidor(direccion_servidor)?;

        //Iniciar la comunicacion con el servidor
        // obtener_listas_de_commits
        self.iniciar_git_upload_pack_con_servidor(&mut comunicacion)?;

        let mut refs_recibidas = comunicacion.obtener_lineas()?;

        let capacidades = self.fase_de_descubrimiento(&mut refs_recibidas);
        // envio
        let wants = comunicacion
            .obtener_wants_pkt(&refs_recibidas, capacidades.to_string())
            .unwrap();
        comunicacion.responder(wants.clone()).unwrap();

        let objetos_directorio =
            io::obtener_objetos_del_directorio("./.gir/objects/".to_string()).unwrap();
        let haves = comunicacion.obtener_haves_pkt(&objetos_directorio);
        if !haves.is_empty() {
            comunicacion.responder(haves).unwrap();
            let acks_nak = comunicacion.obtener_lineas().unwrap();
            println!("acks_nack: {:?}", acks_nak);
            comunicacion
                .responder(vec![io::obtener_linea_con_largo_hex("done")])
                .unwrap();
        } else {
            comunicacion
                .responder(vec![io::obtener_linea_con_largo_hex("done")])
                .unwrap();
            let acks_nak = comunicacion.obtener_lineas().unwrap();
            println!("acks_nack: {:?}", acks_nak);
        }

        // aca para git daemon hay que poner un recibir linea mas porque envia un ACK repetido (No entiendo por que...)
        println!("Obteniendo paquete..");
        let mut packfile = comunicacion.obtener_lineas_como_bytes().unwrap();
        comunicacion
            .obtener_paquete_y_escribir(&mut packfile, String::from("./.gir/objects/"))
            .unwrap();
        Ok(String::from("Fetch ejecutado con exito"))
    }

    fn fase_de_descubrimiento(&mut self, refs_recibidas: &mut Vec<String>) -> &str {
        let first_ref = refs_recibidas.remove(0);
        self.actualizar_ramas_locales_del_remoto(&*refs_recibidas);

        let capacidades = first_ref.split("\0").collect::<Vec<&str>>()[1];
        capacidades
    }

    fn iniciar_git_upload_pack_con_servidor(&self, comunicacion: &mut Comunicacion) -> Result<(), String> {
        let data_del_pedido = "git-upload-pack /.gir/\0host=example.com\0\0version=1\0";
        comunicacion.enviar(&io::obtener_linea_con_largo_hex(data_del_pedido))?;
        Ok(())
    }

    fn convertir_de_dir_rama_remota_a_dir_rama_local(
        &self,
        dir_rama_remota: &str,
    ) -> Result<PathBuf, String> {
        let carpeta_del_remoto = format!("./.gir/refs/remotes/{}/", self.remoto);
        //"./.gir/refs/remotes/origin/";

        let rama_remota = utilidades_path_buf::obtener_nombre(&PathBuf::from(dir_rama_remota))?;
        let dir_rama_local = PathBuf::from(carpeta_del_remoto + rama_remota.as_str());

        Ok(dir_rama_local)
    }

    fn obtener_commits_y_ramas_a_actulizar(
        &self,
        referencia: &String,
    ) -> Result<(&str, &str), String> {
        let (commit_cabeza_de_rama, dir_rama_remota) = referencia.split_once(' ').ok_or(
            format!("Fallo al separar el conendio en actualizar referencias\n"),
        )?;
        Ok((commit_cabeza_de_rama, dir_rama_remota))
    }

    ///actuliza a donde apuntan las ca
    fn actualizar_ramas_locales_del_remoto(&self, referencias: &Vec<String>) -> Result<(), String> {
        for referencia in referencias {
            let (commit_cabeza_de_rama, dir_rama_remota) =
                self.obtener_commits_y_ramas_a_actulizar(referencia)?;

            let dir_rama_local_del_remoto =
                self.convertir_de_dir_rama_remota_a_dir_rama_local(dir_rama_remota)?;

            println!("Voy a escribir en: {:?}", dir_rama_local_del_remoto);
            io::escribir_bytes(dir_rama_local_del_remoto, commit_cabeza_de_rama)?;
        }

        Ok(())
    }
}


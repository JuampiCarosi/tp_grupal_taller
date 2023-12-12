use std::{
    io::{Read, Write},
    path::PathBuf,
    sync::Arc,
};

use crate::tipos_de_dato::{
    comando::Ejecutar,
    comandos::{branch::Branch, init::Init, remote::Remote},
    logger::Logger,
};

use super::io;

pub struct MockTcpStream {
    pub lectura_data: Vec<u8>,
    pub escritura_data: Vec<u8>,
}

impl Read for MockTcpStream {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let bytes_to_read = std::cmp::min(buf.len(), self.lectura_data.len());
        buf[..bytes_to_read].copy_from_slice(&self.lectura_data[..bytes_to_read]);
        self.lectura_data.drain(..bytes_to_read);
        Ok(bytes_to_read)
    }
}

impl Write for MockTcpStream {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.escritura_data.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.escritura_data.flush()
    }
}

pub fn limpiar_archivo_gir(logger: Arc<Logger>) {
    elimar_archivo_gir();

    let mut init = Init {
        path: "./.gir".to_string(),
        logger,
    };
    init.ejecutar().unwrap();
}

pub fn anadir_remoto_default_config(remoto: &str, logger: Arc<Logger>) {
    let mut args_remote = vec![
        "add".to_string(),
        remoto.to_string(),
        "ip:puerto/remoto/".to_string(),
    ];
    Remote::from(&mut args_remote, logger)
        .unwrap()
        .ejecutar()
        .unwrap();
}

///crear la carpeta remota de fomra artificial.
///
/// # Resultado
/// - Crear la carteta `./.gir/refs/remotes/{remoto}/{nombre_rama_remota}`
pub fn escribir_rama_remota(remoto: &str, nombre_rama_remota: &str) {
    let dir = format!("./.gir/refs/remotes/{}/{}", remoto, nombre_rama_remota);
    io::escribir_bytes(dir, "contenido").unwrap();
}

pub fn escribir_rama_local(rama: &str, logger: Arc<Logger>) {
    let mut branch_args = vec![rama.to_string()];
    Branch::from(&mut branch_args, logger)
        .unwrap()
        .ejecutar()
        .unwrap();
}

pub fn elimar_archivo_gir() {
    if PathBuf::from("./.gir").exists() {
        io::rm_directorio("./.gir").unwrap();
    }
}

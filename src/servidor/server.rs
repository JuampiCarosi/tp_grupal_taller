use crate::err_comunicacion::ErrorDeComunicacion;
use crate::servidor::{receive_pack::receive_pack, upload_pack::upload_pack};
use crate::tipos_de_dato::respuesta_pedido::RespuestaDePedido;
use crate::tipos_de_dato::{comunicacion::Comunicacion, logger::Logger};
use crate::utils::{self, io as gir_io};
use std::{
    env,
    net::{TcpListener, TcpStream},
    path::PathBuf,
    str,
    sync::Arc,
    thread,
};

const VERSION: &str = "version 1\n";
const CAPABILITIES: &str = "ofs-delta symref=HEAD:refs/heads/master agent=git/2.17.1";
const DIR: &str = "/srv"; // direccion relativa

///
pub struct Servidor {
    /// Canal para escuchar las conexiones de clientes
    listener: TcpListener,

    /// Threads que se spawnean para atender a los clientes
    threads: Vec<Option<thread::JoinHandle<Result<(), String>>>>,

    /// Logger para registrar los eventos del servidor
    logger: Arc<Logger>,
}

impl Servidor {
    /// # Argumentos:
    /// * `address` - Direccion en la que se va a escuchar las conexiones de los clientes
    /// * `logger` - Logger para registrar los eventos del servidor
    pub fn new(address: &str, logger: Arc<Logger>) -> std::io::Result<Servidor> {
        let listener = TcpListener::bind(address)?;
        println!("Escuchando en {}", address);
        logger.log("Servidor iniciado");

        Ok(Servidor {
            listener,
            threads: Vec::new(),
            logger,
        })
    }

    /// Pone en funcionamiento el servidor, spawneando un thread por cada cliente que se conecte al mismo.
    /// Procesa el pedido del cliente y responde en consecuencia.
    pub fn iniciar_servidor(&mut self) -> Result<(), String> {
        while let Ok((stream, socket)) = self.listener.accept() {
            self.logger
                .log(&format!("Se conecto un cliente desde {}", socket));
            let logge_clone = self.logger.clone();
            let logger = self.logger.clone();
            let handle = thread::spawn(move || -> Result<(), String> {
                let stream_clonado = stream.try_clone().map_err(|e| e.to_string())?;
                let mut comunicacion =
                    Comunicacion::<TcpStream>::new_para_server(stream_clonado, logge_clone);
                Self::manejar_cliente(
                    &mut comunicacion,
                    &(env!("CARGO_MANIFEST_DIR").to_string() + DIR),
                    logger,
                )?;
                Ok(())
            });
            self.threads.push(Some(handle));
        }
        self.logger.log("Se cerro el servidor");
        Ok(())
    }

    // Funcion para parsear el pedido del cliente y actuar segun corresponda, retorna en caso de que se haya enviado un
    fn manejar_cliente(
        comunicacion: &mut Comunicacion<TcpStream>,
        dir: &str,
        logger: Arc<Logger>,
    ) -> Result<(), String> {
        let pedido = match comunicacion.aceptar_pedido()? {
            RespuestaDePedido::Mensaje(mensaje) => mensaje,
            RespuestaDePedido::Terminate => return Ok(()),
        }; // acepto la primera linea
        Self::procesar_pedido(&pedido, comunicacion, dir, logger)?; // parse de la liena para ver que se pide
        Ok(())
    }

    // Facilita la primera parte de la funcion anterior
    fn parsear_linea_pedido_y_responder_con_version(
        linea_pedido: &str,
        dir: &str,
    ) -> Result<(String, String, String), String> {
        let pedido: Vec<String> = linea_pedido
            .split_whitespace()
            .map(|s| s.to_string())
            .collect();

        let args: Vec<String> = pedido[1].split('\0').map(|s| s.to_string()).collect();
        let repositorio = args[0].clone();
        let dir_repositorio = dir.to_string() + &args[0];
        let pedido = &pedido[0];
        Ok((pedido.to_owned(), repositorio, dir_repositorio))
    }

    // Funcion para actuar segun si se recibe un upload-pack o un receive-pack, en caso de que sea un receive-pack y el repositorio no exista, se crea el mismo
    fn procesar_pedido(
        linea: &str,
        comunicacion: &mut Comunicacion<TcpStream>,
        dir: &str,
        logger: Arc<Logger>,
    ) -> Result<(), String> {
        let (pedido, repo, dir_repo) =
            Self::parsear_linea_pedido_y_responder_con_version(linea, dir)?;
        let refs: Vec<String>;
        let resultado_de_ejecucion: Result<(), String>;
        match pedido.as_str() {
            "git-upload-pack" => {
                if !PathBuf::from(&dir_repo).exists() {
                    let error = ErrorDeComunicacion::ErrorRepositorioNoExiste(repo).to_string();
                    comunicacion.enviar(&utils::strings::obtener_linea_con_largo_hex(&error))?;
                    logger.log(&error);
                    return Err("No existe el repositorio".to_string());
                }
                comunicacion.enviar(&utils::strings::obtener_linea_con_largo_hex(
                    &(VERSION.to_string()),
                ))?;
                println!("upload-pack recibido, ejecutando");
                refs = server_utils::obtener_refs_de(PathBuf::from(&dir_repo))?;
                comunicacion.responder(&refs)?;
                resultado_de_ejecucion = upload_pack(dir_repo, comunicacion, &refs, logger.clone());
                if let Err(e) = &resultado_de_ejecucion {
                    logger.log(e);
                }
                resultado_de_ejecucion
            }
            "git-receive-pack" => {
                println!("receive-pack recibido, ejecutando");
                let path = PathBuf::from(&dir_repo);

                if !path.exists() {
                    gir_io::crear_directorio(path.join("refs/"))?;
                    gir_io::crear_directorio(path.join("refs/heads/"))?;
                    gir_io::crear_directorio(path.join("refs/tags/"))?;
                }
                comunicacion.enviar(&utils::strings::obtener_linea_con_largo_hex(
                    &(VERSION.to_string()),
                ))?;
                refs = server_utils::obtener_refs_de(path)?;
                comunicacion.responder(&refs)?;
                resultado_de_ejecucion =
                    receive_pack(dir_repo.to_string(), comunicacion, logger.clone());
                if let Err(e) = &resultado_de_ejecucion {
                    logger.log(e);
                }
                resultado_de_ejecucion
            }
            _ => {
                comunicacion.enviar(&utils::strings::obtener_linea_con_largo_hex(
                    "ERR No existe el comando\n",
                ))?;
                logger.log(&format!("No existe el comando {}", pedido));
                Err("No existe el comando".to_string())
            }
        }
    }
}

// -------------- utils del server --------------
mod server_utils {
    use super::*;

    /// Funcion que busca y devuelve las referencias de una direccion dada en formato pkt de un directorio con el formato de git
    pub fn obtener_refs_de(dir: PathBuf) -> Result<Vec<String>, String> {
        let mut refs: Vec<String> = Vec::new();
        let head_ref = utils::referencia::obtener_ref_head(dir.join("HEAD"));
        if let Ok(head) = head_ref {
            refs.push(head)
        }
        let dir_str = match dir.to_str() {
            Some(s) => s,
            None => return Err("No se pudo convertir el path {dir} a str".to_string()),
        };
        utils::referencia::obtener_refs_con_largo_hex(&mut refs, dir.join("refs/heads/"), dir_str)?;
        utils::referencia::obtener_refs_con_largo_hex(&mut refs, dir.join("refs/tags/"), dir_str)?;
        if !refs.is_empty() {
            let ref_con_cap = agregar_capacidades(refs[0].clone());
            refs.remove(0);
            refs.insert(0, ref_con_cap);
        } else {
            refs.push(agregar_capacidades("0".repeat(40)));
        }
        Ok(refs)
    }

    /// Funcion que agrega las capacidades del servidor a una referencia dada en formato pkt
    pub fn agregar_capacidades(referencia: String) -> String {
        let mut referencia_con_capacidades: String;
        if referencia.len() > 40 {
            referencia_con_capacidades = referencia.split_at(4).1.to_string() + "\0";
        } else {
            referencia_con_capacidades = referencia + "\0";
        }
        let capacidades: Vec<&str> = CAPABILITIES.split_whitespace().collect();
        for cap in capacidades.iter() {
            referencia_con_capacidades.push_str(&format!("{} ", cap));
        }
        let mut referencia_con_capacidades = referencia_con_capacidades.trim_end().to_string();
        referencia_con_capacidades.push('\n');
        utils::strings::obtener_linea_con_largo_hex(&referencia_con_capacidades)
    }
}

impl Drop for Servidor {
    fn drop(&mut self) {
        for thread in self.threads.drain(..).flatten() {
            if let Err(e) = thread.join() {
                println!("Error en el thread: {:?}", e);
            }
        }
        println!("Servidor cerrado");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test01_agregar_capacidades() {
        let referencia = "0".repeat(40);
        let referencia_con_capacidades = server_utils::agregar_capacidades(referencia);
        println!("{}", referencia_con_capacidades);
        assert_eq!(
            referencia_con_capacidades,
            utils::strings::obtener_linea_con_largo_hex(
                &("0".repeat(40).to_string() + "\0" + CAPABILITIES + "\n")
            )
        );
    }

    #[test]
    fn test02_obtener_refs_con_ref_vacia_devuelve_ref_nula() {
        let dir =
            PathBuf::from(env!("CARGO_MANIFEST_DIR").to_string() + "/server_test_dir/test02/.gir/");
        let refs = server_utils::obtener_refs_de(dir).unwrap();
        println!("{:?}", refs);
        assert_eq!(
            refs[0],
            utils::strings::obtener_linea_con_largo_hex(
                &("0".repeat(40).to_string() + "\0" + CAPABILITIES + "\n")
            )
        );
    }

    #[test]
    fn test03_obtener_refs_con_ref_head_devuelve_ref_head() {
        let dir =
            PathBuf::from(env!("CARGO_MANIFEST_DIR").to_string() + "/server_test_dir/test03/.gir/");
        let refs = server_utils::obtener_refs_de(dir).unwrap();
        println!("{:?}", refs);
        assert_eq!(
            refs[0],
            utils::strings::obtener_linea_con_largo_hex(
                &("4163eb28ec61fd1d0c17cf9b77f4c17e1e338b0b".to_string()
                    + " HEAD\0"
                    + CAPABILITIES
                    + "\n")
            )
        );
    }
}

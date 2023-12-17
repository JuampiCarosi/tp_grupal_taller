use std::{
    io::{BufReader, Read, Write},
    net::{TcpListener, TcpStream},
    sync::{mpsc::Sender, Arc, Mutex},
    thread,
};

use crate::{
    tipos_de_dato::{
        http::{
            endpoint::Endpoint, error::ErrorHttp, estado::EstadoHttp, request::Request,
            response::Response,
        },
        logger::Logger,
    },
    utils::gir_config,
};

use super::rutas::{
    actualizar_pull_request, crear_pull_request, listar_pull_request,
    mensaje_servidor::MensajeServidor, mergear_pull_request, obtener_commits_pull_request,
    obtener_pull_request,
};

pub struct ServidorHttp {
    /// Canal para escuchar las conexiones de clientes
    listener: TcpListener,

    /// Logger para registrar los eventos del servidor
    logger: Arc<Logger>,

    main: Option<thread::JoinHandle<()>>,

    threads: Arc<Mutex<Vec<thread::JoinHandle<Result<(), String>>>>>,

    tx: Sender<MensajeServidor>,
}

impl ServidorHttp {
    /// # Argumentos:
    /// * `address` - Direccion en la que se va a escuchar las conexiones de los clientes
    /// * `logger` - Logger para registrar los eventos del servidor
    pub fn new(
        logger: Arc<Logger>,
        threads: Arc<Mutex<Vec<thread::JoinHandle<Result<(), String>>>>>,
        tx: Sender<MensajeServidor>,
    ) -> Result<Self, String> {
        let puerto = gir_config::conseguir_puerto_http()
            .ok_or("No se pudo conseguir el puerto http, revise el archivo config")?;

        let address = "127.0.0.1:".to_owned() + &puerto;

        let listener = TcpListener::bind(&address).map_err(|e| e.to_string())?;
        println!("Escuchando servidor HTTP en {}", address);
        logger.log("Servidor iniciado");

        Ok(Self {
            listener,
            logger,
            threads,
            main: None,
            tx,
        })
    }

    fn agregar_endpoints(endpoints: &mut Vec<Endpoint>) {
        crear_pull_request::agregar_a_router(endpoints);
        listar_pull_request::agregar_a_router(endpoints);
        obtener_pull_request::agregar_a_router(endpoints);
        obtener_commits_pull_request::agregar_a_router(endpoints);
        actualizar_pull_request::agregar_a_router(endpoints);
        mergear_pull_request::agregar_a_router(endpoints);
    }

    fn aceptar_conexiones(
        endpoints: Arc<Vec<Endpoint>>,
        listener: TcpListener,
        threads: Arc<Mutex<Vec<thread::JoinHandle<Result<(), String>>>>>,
        logger: Arc<Logger>,
        tx: Sender<MensajeServidor>,
    ) {
        while let Ok((mut stream, socket)) = listener.accept() {
            logger.log(&format!("Se conecto un cliente por http desde {}", socket));

            let logger_clone = logger.clone();
            let endpoints = endpoints.clone();
            let tx = tx.clone();
            let handle = thread::spawn(move || -> Result<(), String> {
                let response = Self::manejar_cliente(logger_clone.clone(), &mut stream, &endpoints);
                match response {
                    Ok(response) => response.enviar(&mut stream).map_err(|e| e.to_string()),
                    Err(error_http) => {
                        logger_clone.log(&format!("Error procesando request: {:?}", error_http));
                        let response = Response::from_error(logger_clone.clone(), error_http);
                        tx.send(MensajeServidor::HttpErrorFatal)
                            .map_err(|e| e.to_string())?;
                        response.enviar(&mut stream).map_err(|e| e.to_string())
                    }
                }?;

                return Ok(());
            });

            let threads = threads.lock();

            if let Ok(mut threads) = threads {
                threads.push(handle);
            } else {
                logger.log("Error al obtener el lock de threads");
            }
        }
    }

    pub fn reiniciar_servidor(&mut self) -> Result<(), String> {
        self.logger.log("Reiniciando servidor http");
        self.main.take();
        self.iniciar_servidor()
    }

    /// Pone en funcionamiento el servidor, spawneando un thread por cada cliente que se conecte al mismo.
    /// Procesa el pedido del cliente y responde en consecuencia.
    pub fn iniciar_servidor(&mut self) -> Result<(), String> {
        let logger = self.logger.clone();
        let listener = self.listener.try_clone().map_err(|e| e.to_string())?;
        let threads = self.threads.clone();
        let tx = self.tx.clone();
        let main = thread::spawn(|| {
            let mut endpoints = Vec::new();
            Self::agregar_endpoints(&mut endpoints);
            let endpoints = Arc::new(endpoints);
            Self::aceptar_conexiones(endpoints, listener, threads, logger, tx);
        });

        self.main.replace(main);
        Ok(())
    }

    fn manejar_cliente<R: Read + Write>(
        logger: Arc<Logger>,
        stream: &mut R,
        endpoints: &Vec<Endpoint>,
    ) -> Result<Response, ErrorHttp> {
        // let mut stream_clone = stream
        //     .clone()
        //     .map_err(|e| ErrorHttp::InternalServerError(e.to_string()))?;

        let mut reader = BufReader::new(stream);
        let request = Request::from(&mut reader, logger.clone())?;
        for endpoint in endpoints {
            if endpoint.metodo != request.metodo {
                continue;
            }

            let params = match endpoint.matchea_con_patron(&request.ruta) {
                Some(params) => params,
                None => continue,
            };
            let response = (endpoint.handler)(request, params, logger.clone())?;

            return Ok(response);
        }

        let response = Response::new(logger, EstadoHttp::NotFound, None);
        Ok(response)
    }
}


#[cfg(test)]
mod test {
    use std::path::PathBuf;

    use super::*;
    use crate::{utils::{testing::{self, crear_repo_para_pr}, io}, servidor::gir_server::ServidorGir};
    #[test]
    fn test01_se_obtiene_not_found_si_no_existe_el_repositorio() {
        let contenido_mock = "GET /repos/{owner}/{repo}/pulls/{pull_number} HTTP/1.1\r\n\r\n";
        let logger = Arc::new(Logger::new(PathBuf::from("server_logger.txt")).unwrap());
        let mut mock = testing::MockTcpStream {
            lectura_data: contenido_mock.as_bytes().to_vec(),
            escritura_data: vec![],
        };

        let respuesta = ServidorHttp::manejar_cliente(
            logger.clone(),
            &mut mock,
            &vec![],
        ).unwrap();

        assert_eq!(404, respuesta.estado);
        assert_eq!("Not Found", respuesta.mensaje_estado);
    }

    #[test]
    fn test02_crear_pr_en_repo_devuelve_status_201() {
        
        let logger = Arc::new(Logger::new(PathBuf::from("tmp/servidor_http_test02")).unwrap());

        let logger_clone = logger.clone();
        let (tx, _) = std::sync::mpsc::channel();

        let handle = std::thread::spawn(move || {
            let threads = Arc::new(Mutex::new(Vec::new()));
            let listener = TcpListener::bind("127.0.0.1:9933").unwrap();

            let mut servidor_gir = ServidorGir {
                listener,
                threads,
                logger: logger_clone,
                main: None,
                tx,
            };
            servidor_gir.iniciar_servidor().unwrap();
        });

        if handle.is_finished() {
            panic!("No se pudo iniciar el servidor");
        }
        std::thread::sleep(std::time::Duration::from_secs(1));

        let _ = io::rm_directorio("tmp/servidor_http_test02_dir");
        let _ = io::rm_directorio("srv/repo/");
        io::crear_directorio("tmp/servidor_http_test02_dir").unwrap();
        io::cambiar_directorio("tmp/servidor_http_test02_dir").unwrap();

        crear_repo_para_pr(logger.clone());
        std::thread::sleep(std::time::Duration::from_secs(1));
        
        let repo = "repo";
        let body = r#"{
            "title": "Feature X: Implement new functionality",
            "head": "juani:master",
            "base": "rama",
            "body": "This is the description of the pull request."
        }"#;
        let content_length = body.len();

        let request_string = format!(
            "POST /repos/{}/pulls HTTP/1.1\r\n\
            Host: localhost:9933\r\n\
            Accept: application/vnd.github+json\r\n\
            Content-Type: application/json\r\n\
            Content-Length: {}\r\n\
            \r\n\
            {}",
            repo,
            content_length,
            body
        );

        let mut mock = testing::MockTcpStream {
            lectura_data: request_string.as_bytes().to_vec(),
            escritura_data: vec![],
        };
        io::cambiar_directorio("../../").unwrap();
        let mut endpoints = Vec::new();
        ServidorHttp::agregar_endpoints(&mut endpoints);

        let respuesta = ServidorHttp::manejar_cliente(
            logger.clone(),
            &mut mock,
            &endpoints,
        ).unwrap();
        io::rm_directorio("tmp/servidor_http_test02_dir").unwrap();
        io::rm_directorio("srv/repo/").unwrap();
        assert_eq!(201, respuesta.estado);
        assert_eq!("Created", respuesta.mensaje_estado);  
    }

}
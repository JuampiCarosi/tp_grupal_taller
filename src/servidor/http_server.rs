use std::{
    io::BufReader,
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

            let mut threads = threads.lock().unwrap();
            threads.push(handle);
        }
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

    fn manejar_cliente(
        logger: Arc<Logger>,
        stream: &mut TcpStream,
        endpoints: &Vec<Endpoint>,
    ) -> Result<Response, ErrorHttp> {
        let mut stream_clone = stream
            .try_clone()
            .map_err(|e| ErrorHttp::InternalServerError(e.to_string()))?;

        let mut reader = BufReader::new(&mut stream_clone);
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

// impl Drop for ServidorHttp {
//     fn drop(&mut self) {
//         for handle in self.threads.iter_mut() {
//             if let Some(handle) = handle.take() {
//                 match handle.join() {
//                     Ok(_) => (),
//                     Err(_) => self.logger.log("Error en un thread"),
//                 };
//             }
//         }
//         self.logger.log("Cerrando servidor");
//     }
// }

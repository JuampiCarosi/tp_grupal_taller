use std::{
    io::BufReader,
    net::{TcpListener, TcpStream},
    sync::Arc,
    thread,
};

use crate::tipos_de_dato::{
    http_request::HttpRequest, http_response::HttpResponse, logger::Logger,
};

pub struct ServidorHttp {
    /// Canal para escuchar las conexiones de clientes
    listener: TcpListener,

    /// Threads que se spawnean para atender a los clientes
    threads: Vec<Option<thread::JoinHandle<Result<(), String>>>>,

    /// Logger para registrar los eventos del servidor
    logger: Arc<Logger>,
}

impl ServidorHttp {
    /// # Argumentos:
    /// * `address` - Direccion en la que se va a escuchar las conexiones de los clientes
    /// * `logger` - Logger para registrar los eventos del servidor
    pub fn new(address: &str, logger: Arc<Logger>) -> std::io::Result<Self> {
        let listener = TcpListener::bind(address)?;
        println!("Escuchando servidor HTTP en {}", address);
        logger.log("Servidor iniciado");

        Ok(Self {
            listener,
            threads: Vec::new(),
            logger,
        })
    }

    /// Pone en funcionamiento el servidor, spawneando un thread por cada cliente que se conecte al mismo.
    /// Procesa el pedido del cliente y responde en consecuencia.
    pub fn iniciar_servidor(&mut self) -> Result<(), String> {
        while let Ok((mut stream, socket)) = self.listener.accept() {
            self.logger
                .log(&format!("Se conecto un cliente por http desde {}", socket));

            let logge_clone = self.logger.clone();
            let handle = thread::spawn(move || -> Result<(), String> {
                Self::manejar_cliente(logge_clone.clone(), &mut stream)?;
                Ok(())
            });
            self.threads.push(Some(handle));
        }
        self.logger.log("Se cerro el servidor");
        Ok(())
    }

    fn manejar_cliente(logger: Arc<Logger>, stream: &mut TcpStream) -> Result<(), String> {
        // Read headers
        let mut stream_clone = stream.try_clone().map_err(|e| e.to_string())?;
        let mut reader = BufReader::new(&mut stream_clone);

        let request = HttpRequest::from(&mut reader, logger.clone());

        match request {
            Ok(request) => {
                logger.log(&format!("Request: {:?}", request));
                let response = HttpResponse::new_ok(logger.clone(), Some(request.body.clone()));
                response.enviar(stream)?;
            }
            Err(err) => {
                logger.log(&format!("Error: {}", err));
                let response = HttpResponse::new_not_found(logger.clone());
                response.enviar(stream)?;
            }
        }

        Ok(())
    }
}

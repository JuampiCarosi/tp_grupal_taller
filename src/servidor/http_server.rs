use std::{
    io::Read,
    net::{TcpListener, TcpStream},
    sync::Arc,
    thread,
};

use crate::tipos_de_dato::{
    comunicacion::Comunicacion, logger::Logger, respuesta_pedido::RespuestaDePedido,
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
        let mut buf: [u8; 1024] = [0; 1024];
        let _req = stream.read(&mut buf).map_err(|e| e.to_string())?;

        println!("Request: {:?}", String::from_utf8_lossy(&buf[..]));

        Ok(())
    }
}

use std::{collections::HashMap, io::Write, net::TcpStream, sync::Arc};

use super::{estado_http::EstadoHttp, logger::Logger};

pub struct HttpResponse {
    pub estado: usize,
    pub mensaje_estado: String,
    pub version: String,
    pub headers: HashMap<String, String>,
    pub body: Option<String>,
    pub logger: Arc<Logger>,
}

impl HttpResponse {
    pub fn new(logger: Arc<Logger>, estado: EstadoHttp, body: Option<&str>) -> Self {
        let mut headers: HashMap<String, String> = HashMap::new();
        if let Some(body) = &body {
            headers.insert("Content-lenght".to_string(), body.len().to_string());
            headers.insert("Content-Type".to_string(), "application/json".to_string());
        }

        let (estado, mensaje_estado) = estado.obtener_estado_y_mensaje();

        Self {
            estado,
            mensaje_estado,
            version: "HTTP/1.1".to_string(),
            headers: HashMap::new(),
            body: body.map(|s| s.to_string()),
            logger,
        }
    }

    pub fn enviar(&self, stream: &mut TcpStream) -> Result<(), String> {
        let mut response = format!(
            "{version} {estado} {mensaje_estado}\r\n",
            version = self.version,
            estado = self.estado,
            mensaje_estado = self.mensaje_estado
        );

        for (key, value) in &self.headers {
            response.push_str(&format!("{}: {}\r\n", key, value));
        }
        response.push_str("\r\n");

        if let Some(body) = &self.body {
            response.push_str(body);
        }

        stream.write_all(response.as_bytes()).unwrap();

        Ok(())
    }
}

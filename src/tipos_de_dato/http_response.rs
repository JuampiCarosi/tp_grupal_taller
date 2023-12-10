use std::{collections::HashMap, io::Write, net::TcpStream, sync::Arc};

use super::logger::Logger;

pub struct HttpResponse {
    pub estado: usize,
    pub mensaje_estado: String,
    pub version: String,
    pub headers: HashMap<String, String>,
    pub body: Option<String>,
    pub logger: Arc<Logger>,
}

impl HttpResponse {
    pub fn new_ok(logger: Arc<Logger>, body: Option<String>) -> Self {
        let mut headers: HashMap<String, String> = HashMap::new();
        if let Some(body) = &body {
            headers.insert("Content-lenght".to_string(), body.len().to_string());
        }

        Self {
            estado: 200,
            mensaje_estado: "OK".to_string(),
            version: "HTTP/1.1".to_string(),
            headers: HashMap::new(),
            body,
            logger,
        }
    }

    pub fn new_not_found(logger: Arc<Logger>) -> Self {
        Self {
            estado: 404,
            mensaje_estado: "Not Found".to_string(),
            version: "HTTP/1.1".to_string(),
            headers: HashMap::new(),
            body: None,
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

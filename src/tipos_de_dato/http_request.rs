use std::{
    collections::HashMap,
    fmt::{Debug, Formatter},
    io::{BufRead, BufReader, Read},
    net::TcpStream,
    sync::Arc,
};

use super::logger::Logger;

pub struct HttpRequest {
    pub method: String,
    pub path: String,
    pub version: String,
    pub headers: HashMap<String, String>,
    pub body: String,
    pub logger: Arc<Logger>,
}

impl HttpRequest {
    pub fn from(
        reader: &mut BufReader<&mut TcpStream>,
        logger: Arc<Logger>,
    ) -> Result<Self, String> {
        let start_line = Self::obtener_primera_linea(reader)?;

        let headers = Self::obtener_headers(reader)?;

        let content_length = headers
            .get("Content-Length")
            .ok_or("No Content-Length header")?
            .parse::<usize>()
            .unwrap_or(0);

        let body = Self::obtener_body(reader, content_length)?;

        let splitted = start_line.split_whitespace().collect::<Vec<&str>>();
        if splitted.len() != 3 {
            return Err("Invalid start line".to_string());
        }

        let method = splitted[0].to_string();
        let path = splitted[1].to_string();

        if &path != "/" {
            return Err("not found".to_string());
        }

        let version = splitted[2].to_string();

        Ok(Self {
            method,
            path,
            version,
            headers,
            body,
            logger,
        })
    }

    fn obtener_headers(
        reader: &mut BufReader<&mut TcpStream>,
    ) -> Result<HashMap<String, String>, String> {
        let mut headers = HashMap::new();

        loop {
            let mut line = String::new();
            reader.read_line(&mut line).unwrap();
            if line == "\r\n" {
                break;
            }
            let splitted = line.splitn(2, ":").collect::<Vec<&str>>();
            if splitted.len() != 2 {
                return Err("Invalid header".to_string());
            }

            let key = splitted[0].trim().to_string();
            let value = splitted[1].trim().to_string();

            headers.insert(key, value);
        }

        Ok(headers)
    }

    fn obtener_primera_linea(reader: &mut BufReader<&mut TcpStream>) -> Result<String, String> {
        let mut line = String::new();
        reader.read_line(&mut line).unwrap();
        Ok(line.trim().to_string())
    }

    fn obtener_body(
        reader: &mut BufReader<&mut TcpStream>,
        largo: usize,
    ) -> Result<String, String> {
        let mut body_buf = vec![0; largo];
        reader.read_exact(&mut body_buf).unwrap();
        let body = String::from_utf8_lossy(&body_buf);

        Ok(body.to_string())
    }
}

impl Debug for HttpRequest {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HttpRequest")
            .field("method", &self.method)
            .field("path", &self.path)
            .field("version", &self.version)
            .field("headers", &self.headers)
            .field("body", &self.body)
            .finish()
    }
}

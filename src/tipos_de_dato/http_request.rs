use std::{
    collections::HashMap,
    fmt::{Debug, Formatter},
    io::{BufRead, BufReader, Read},
    net::TcpStream,
    sync::Arc,
};

use super::{estado_http::EstadoHttp, logger::Logger};

pub struct HttpRequest {
    pub metodo: String,
    pub ruta: String,
    pub version: String,
    pub headers: HashMap<String, String>,
    pub body: Option<String>,
    pub logger: Arc<Logger>,
}

impl HttpRequest {
    fn obtener_content_length(
        headers: &HashMap<String, String>,
    ) -> Result<Option<usize>, EstadoHttp> {
        let raw = headers.get("Content-Length");

        if let Some(raw) = raw {
            match raw.parse::<usize>() {
                Ok(largo) => return Ok(Some(largo)),
                Err(_) => return Err(EstadoHttp::BadRequest),
            };
        }

        Ok(None)
    }

    pub fn from(
        reader: &mut BufReader<&mut TcpStream>,
        logger: Arc<Logger>,
    ) -> Result<Self, EstadoHttp> {
        let (metodo, ruta, version) = Self::obtener_primera_linea(reader)?;

        let headers = Self::obtener_headers(reader)?;
        let content_length = Self::obtener_content_length(&headers)?;
        let body = Self::obtener_body(reader, content_length)?;

        Ok(Self {
            metodo,
            ruta,
            version,
            headers,
            body,
            logger,
        })
    }

    fn obtener_headers(
        reader: &mut BufReader<&mut TcpStream>,
    ) -> Result<HashMap<String, String>, EstadoHttp> {
        let mut headers = HashMap::new();

        loop {
            let mut line = String::new();
            reader.read_line(&mut line).unwrap();
            if line == "\r\n" {
                break;
            }
            let splitted = line.splitn(2, ":").collect::<Vec<&str>>();
            if splitted.len() != 2 {
                return Err(EstadoHttp::BadRequest);
            }

            let key = splitted[0].trim().to_string();
            let value = splitted[1].trim().to_string();

            headers.insert(key, value);
        }

        Ok(headers)
    }

    fn obtener_primera_linea(
        reader: &mut BufReader<&mut TcpStream>,
    ) -> Result<(String, String, String), EstadoHttp> {
        let mut line = String::new();
        reader.read_line(&mut line).unwrap();

        let splitted = line.split_whitespace().collect::<Vec<&str>>();
        if splitted.len() != 3 {
            return Err(EstadoHttp::BadRequest);
        }

        let metodo = splitted[0].to_string();
        let ruta = splitted[1].to_string();
        let version = splitted[2].to_string();

        Ok((metodo, ruta, version))
    }

    fn obtener_body(
        reader: &mut BufReader<&mut TcpStream>,
        largo: Option<usize>,
    ) -> Result<Option<String>, EstadoHttp> {
        let largo = match largo {
            Some(largo) => largo,
            None => return Ok(None),
        };

        let mut body_buf = vec![0; largo];
        reader
            .read_exact(&mut body_buf)
            .map_err(|_| EstadoHttp::BadRequest)?;
        let body = String::from_utf8_lossy(&body_buf);

        Ok(Some(body.to_string()))
    }
}

impl Debug for HttpRequest {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HttpRequest")
            .field("metodo", &self.metodo)
            .field("ruta", &self.ruta)
            .field("version", &self.version)
            .field("headers", &self.headers)
            .field("body", &self.body)
            .finish()
    }
}

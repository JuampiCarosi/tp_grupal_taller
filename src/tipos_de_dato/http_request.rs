use std::{
    collections::HashMap,
    fmt::{Debug, Formatter},
    io::{BufRead, BufReader, Read},
    net::TcpStream,
    sync::Arc,
};

use super::{
    error_http::ErrorHttp, logger::Logger, metodos_http::MetodoHttp,
    tipo_contenido_http::TipoContenidoHttp,
};

pub struct HttpRequest {
    pub metodo: MetodoHttp,
    pub ruta: String,
    pub version: String,
    pub headers: HashMap<String, String>,
    pub body: Option<HashMap<String, String>>,
    pub logger: Arc<Logger>,
}

impl HttpRequest {
    fn parsear_header_largo(option_largo: Option<&String>) -> Result<usize, ErrorHttp> {
        match option_largo {
            Some(largo_raw) => match largo_raw.parse::<usize>() {
                Ok(largo) => Ok(largo),
                Err(e) => Err(ErrorHttp::BadRequest(e.to_string())),
            },
            None => Err(ErrorHttp::BadRequest(
                "No se encontro el header Content-Length".to_string(),
            )),
        }
    }

    fn parsear_header_tipo(option_tipo: Option<&String>) -> Result<TipoContenidoHttp, ErrorHttp> {
        match option_tipo {
            Some(tipo_raw) => {
                let tipo = TipoContenidoHttp::from_string(tipo_raw)
                    .map_err(|e| ErrorHttp::BadRequest(e.to_string()))?;
                Ok(tipo)
            }
            None => Err(ErrorHttp::BadRequest(
                "No se encontro el header Content-Type".to_string(),
            )),
        }
    }

    fn obtener_headers_contenido(
        headers: &HashMap<String, String>,
    ) -> Result<Option<(usize, TipoContenidoHttp)>, ErrorHttp> {
        let option_largo = headers.get("Content-Length");
        let option_tipo = headers.get("Content-Type");

        if option_largo.is_none() && option_tipo.is_none() {
            return Ok(None);
        }

        let largo = Self::parsear_header_largo(option_largo)?;
        if largo == 0 {
            return Ok(None);
        }

        let tipo = Self::parsear_header_tipo(option_tipo)?;

        Ok(Some((largo, tipo)))
    }

    pub fn from(
        reader: &mut BufReader<&mut TcpStream>,
        logger: Arc<Logger>,
    ) -> Result<Self, ErrorHttp> {
        let (metodo, ruta, version) = Self::obtener_primera_linea(reader)?;

        let metodo = MetodoHttp::from_string(&metodo)?;

        let headers = Self::obtener_headers(reader)?;
        let body = Self::obtener_body(reader, &headers)?;

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
    ) -> Result<HashMap<String, String>, ErrorHttp> {
        let mut headers = HashMap::new();

        loop {
            let mut line = String::new();
            reader.read_line(&mut line).unwrap();
            if line == "\r\n" {
                break;
            }
            let splitted = line.splitn(2, ":").collect::<Vec<&str>>();
            if splitted.len() != 2 {
                return Err(ErrorHttp::BadRequest("Error parseando headers".to_string()));
            }

            let key = splitted[0].trim().to_string();
            let value = splitted[1].trim().to_string();

            headers.insert(key, value);
        }

        Ok(headers)
    }

    fn obtener_primera_linea(
        reader: &mut BufReader<&mut TcpStream>,
    ) -> Result<(String, String, String), ErrorHttp> {
        let mut line = String::new();
        reader.read_line(&mut line).unwrap();

        let splitted = line.split_whitespace().collect::<Vec<&str>>();
        if splitted.len() != 3 {
            return Err(ErrorHttp::BadRequest(
                "Error parseando primera linea".to_string(),
            ));
        }

        let metodo = splitted[0].to_string();
        let ruta = splitted[1].to_string();
        let version = splitted[2].to_string();

        Ok((metodo, ruta, version))
    }

    fn obtener_body(
        reader: &mut BufReader<&mut TcpStream>,
        headers: &HashMap<String, String>,
    ) -> Result<Option<HashMap<String, String>>, ErrorHttp> {
        let headers = Self::obtener_headers_contenido(&headers)?;

        let (largo, tipo) = match headers {
            Some((largo, tipo)) => (largo, tipo),
            None => return Ok(None),
        };

        let mut body_buf = vec![0; largo];
        reader
            .read_exact(&mut body_buf)
            .map_err(|e| ErrorHttp::BadRequest(e.to_string()))?;

        let body = tipo.parsear_contenido(&body_buf)?;

        Ok(Some(body))
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

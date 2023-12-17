use std::{
    collections::HashMap,
    io::{Read, Write},
    sync::Arc,
};

use super::{error::ErrorHttp, estado::EstadoHttp, tipo_contenido::TipoContenido};
use crate::tipos_de_dato::logger::Logger;

pub struct Response {
    pub estado: usize,
    pub mensaje_estado: String,
    pub version: String,
    pub headers: HashMap<String, String>,
    pub body: Option<String>,
    pub logger: Arc<Logger>,
}

impl Response {
    pub fn from_error(logger: Arc<Logger>, error: ErrorHttp) -> Result<Self, ErrorHttp> {
        let msj_error = error.obtener_mensaje();
        let body = match !msj_error.is_empty() {
            true => Some(msj_error),
            false => None,
        };

        Response::new(
            logger.clone(),
            error.obtener_estado(),
            body.as_deref(),
            TipoContenido::TextPlain,
        )
    }

    pub fn new(
        logger: Arc<Logger>,
        estado: EstadoHttp,
        body: Option<&str>,
        tipo_contenido: TipoContenido,
    ) -> Result<Self, ErrorHttp> {
        let mut headers: HashMap<String, String> = HashMap::new();

        if let Some(body) = &body {
            headers.insert("Content-Lenght".to_string(), body.len().to_string());
            let contenido = Self::obtener_tipo_contenido(tipo_contenido)?;
            headers.insert("Content-Type".to_string(), contenido);
        }

        let (estado, mensaje_estado) = estado.obtener_estado_y_mensaje();

        Ok(Self {
            estado,
            mensaje_estado,
            version: "HTTP/1.1".to_string(),
            headers,
            body: body.map(|s| s.to_string()),
            logger,
        })
    }

    fn obtener_tipo_contenido(tipo_contenido: TipoContenido) -> Result<String, ErrorHttp> {
        tipo_contenido
            .to_string()
            .map_err(|e| ErrorHttp::InternalServerError(e))
    }

    pub fn enviar<T>(&self, stream: &mut T) -> Result<(), ErrorHttp>
    where
        T: Read + Write,
    {
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

        stream.write_all(response.as_bytes()).map_err(|e| {
            ErrorHttp::InternalServerError(format!("Error al enviar la respuesta: {}", e))
        })?;

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use std::{collections::HashMap, path::PathBuf, sync::Arc};

    use crate::{
        tipos_de_dato::{
            http::{error::ErrorHttp, estado::EstadoHttp, tipo_contenido::TipoContenido},
            logger::Logger,
        },
        utils::testing::MockTcpStream,
    };

    use super::Response;

    #[test]

    fn test_01_new_sin_body() {
        let logger = Arc::new(Logger::new(PathBuf::from("tmp/response_test01")).unwrap());
        let estado = EstadoHttp::Ok;

        let (estado_esparado, mensaje_esperado) = estado.obtener_estado_y_mensaje();
        let verison_esperada = "HTTP/1.1".to_string();

        let response = Response::new(logger, estado, None, TipoContenido::Json).unwrap();

        assert_eq!(response.estado, estado_esparado);
        assert_eq!(response.mensaje_estado, mensaje_esperado);
        assert_eq!(response.version, verison_esperada);
        assert_eq!(response.headers, HashMap::new());
        assert_eq!(response.body, None);
    }

    #[test]

    fn test_02_new_con_body() {
        let logger = Arc::new(Logger::new(PathBuf::from("tmp/response_test02")).unwrap());
        let estado = EstadoHttp::Ok;
        let contenido_body = "Body bien body";
        let body = Some(contenido_body);

        let (estado_esparado, mensaje_esperado) = estado.obtener_estado_y_mensaje();
        let verison_esperada = "HTTP/1.1".to_string();
        let mut header_esperado = HashMap::new();
        header_esperado.insert(
            "Content-Lenght".to_string(),
            contenido_body.len().to_string(),
        );
        header_esperado.insert("Content-Type".to_string(), "application/json".to_string());

        let response = Response::new(logger, estado, body, TipoContenido::Json).unwrap();

        assert_eq!(response.estado, estado_esparado);
        assert_eq!(response.mensaje_estado, mensaje_esperado);
        assert_eq!(response.version, verison_esperada);
        assert_eq!(response.headers, header_esperado);
        assert_eq!(response.body, Some(contenido_body.to_string()));
    }

    #[test]
    fn test_03_se_envia_bien_un_msj_sin_body() {
        let logger = Arc::new(Logger::new(PathBuf::from("tmp/response_test02")).unwrap());

        let mut mock_tcp = MockTcpStream {
            lectura_data: vec![],
            escritura_data: vec![],
        };

        let estado = EstadoHttp::Ok;
        let (estado_esparado, mensaje_esperado) = estado.obtener_estado_y_mensaje();
        let verison_esperada = "HTTP/1.1".to_string();

        Response::new(logger, estado, None, TipoContenido::Json)
            .unwrap()
            .enviar(&mut mock_tcp)
            .unwrap();

        let respuesta_esperada = format!(
            "\
        {verison_esperada} {estado_esparado} {mensaje_esperado}\r\n\r\n"
        );

        assert_eq!(
            mock_tcp.escritura_data.to_owned().as_slice(),
            respuesta_esperada.as_bytes()
        );
    }
    #[test]
    fn test_04_se_envia_bien_un_msj_con_body() {
        let logger = Arc::new(Logger::new(PathBuf::from("tmp/response_test02")).unwrap());

        let mut mock_tcp = MockTcpStream {
            lectura_data: vec![],
            escritura_data: vec![],
        };

        let estado = EstadoHttp::Ok;
        let contenido_body = "Body bien body";
        let body = Some(contenido_body);

        let (estado_esparado, mensaje_esperado) = estado.obtener_estado_y_mensaje();
        let verison_esperada = "HTTP/1.1".to_string();

        Response::new(logger, estado, body, TipoContenido::Json)
            .unwrap()
            .enviar(&mut mock_tcp)
            .unwrap();

        let respuesta_esperada = format!(
            "\
        {verison_esperada} {estado_esparado} {mensaje_esperado}\r\n\
        Content-Lenght: {}\r\n\
        Content-Type: application/json\r\n\
        \r\n\
        {contenido_body}",
            contenido_body.len()
        );

        assert_eq!(
            String::from_utf8_lossy(&mock_tcp.escritura_data).trim(),
            respuesta_esperada.trim(),
        );
    }

    #[test]

    fn test_05_new_desde_error_con_msj() {
        let logger = Arc::new(Logger::new(PathBuf::from("tmp/response_test03")).unwrap());
        let msj_err = "Que rompimos".to_string();
        let error = ErrorHttp::BadRequest(msj_err.clone());

        let (estado_esparado, mensaje_esperado) = error.obtener_estado().obtener_estado_y_mensaje();
        let verison_esperada = "HTTP/1.1".to_string();
        let mut header_esperado = HashMap::new();
        header_esperado.insert("Content-Lenght".to_string(), msj_err.len().to_string());
        header_esperado.insert("Content-Type".to_string(), "text/plain".to_string());

        let response = Response::from_error(logger, error).unwrap();

        assert_eq!(response.estado, estado_esparado);
        assert_eq!(response.mensaje_estado, mensaje_esperado);
        assert_eq!(response.version, verison_esperada);
        assert_eq!(response.headers, header_esperado);
        assert_eq!(response.body, Some(msj_err.to_string()));
    }

    #[test]

    fn test_06_new_desde_error_sin_msj() {
        let logger = Arc::new(Logger::new(PathBuf::from("tmp/response_test03")).unwrap());
        let error = ErrorHttp::BadRequest("".to_string());

        let (estado_esparado, mensaje_esperado) = error.obtener_estado().obtener_estado_y_mensaje();
        let verison_esperada = "HTTP/1.1".to_string();
        let header_esperado = HashMap::new();
        let response = Response::from_error(logger, error).unwrap();

        assert_eq!(response.estado, estado_esparado);
        assert_eq!(response.mensaje_estado, mensaje_esperado);
        assert_eq!(response.version, verison_esperada);
        assert_eq!(response.headers, header_esperado);
        assert_eq!(response.body, None);
    }
}

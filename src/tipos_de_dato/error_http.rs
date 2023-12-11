use super::estado_http::EstadoHttp;

#[derive(Debug)]
pub enum ErrorHttp {
    BadRequest(String),
    NotFound(String),
    InternalServerError(String),
}

impl ErrorHttp {
    pub fn obtener_estado(&self) -> EstadoHttp {
        match self {
            Self::BadRequest(_) => EstadoHttp::BadRequest,
            Self::NotFound(_) => EstadoHttp::NotFound,
            Self::InternalServerError(_) => EstadoHttp::InternalServerError,
        }
    }

    pub fn obtener_mensaje(&self) -> String {
        match self {
            Self::BadRequest(mensaje) => mensaje.to_string(),
            Self::NotFound(mensaje) => mensaje.to_string(),
            Self::InternalServerError(mensaje) => mensaje.to_string(),
        }
    }
}

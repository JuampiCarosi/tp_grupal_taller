use super::estado::EstadoHttp;

#[derive(Debug)]
pub enum ErrorHttp {
    BadRequest(String),
    NotFound(String),
    InternalServerError(String),
    ValidationFailed(String),
}

impl ToString for ErrorHttp {
    fn to_string(&self) -> String {
        match self {
            Self::BadRequest(mensaje) => format!("400 Bad Request: {}", mensaje),
            Self::NotFound(mensaje) => format!("404 Not Found: {}", mensaje),
            Self::InternalServerError(mensaje) => format!("500 Internal Server Error: {}", mensaje),
            Self::ValidationFailed(mensaje) => format!("422 Validation Failed: {}", mensaje),
        }
    }
}

impl ErrorHttp {
    pub fn obtener_estado(&self) -> EstadoHttp {
        match self {
            Self::BadRequest(_) => EstadoHttp::BadRequest,
            Self::NotFound(_) => EstadoHttp::NotFound,
            Self::InternalServerError(_) => EstadoHttp::InternalServerError,
            Self::ValidationFailed(_) => EstadoHttp::ValidationFailed,
        }
    }

    pub fn obtener_mensaje(&self) -> String {
        match self {
            Self::BadRequest(mensaje) => mensaje.to_string(),
            Self::NotFound(mensaje) => mensaje.to_string(),
            Self::InternalServerError(mensaje) => mensaje.to_string(),
            Self::ValidationFailed(mensaje) => mensaje.to_string(),
        }
    }
}

use super::estado::EstadoHttp;

#[derive(Debug)]
pub enum ErrorHttp {
    NotFound(String),
    InternalServerError(String),
    ValidationFailed(String),
    Forbidden(String),
    BadRequest(String),
}

impl ToString for ErrorHttp {
    fn to_string(&self) -> String {
        match self {
            Self::NotFound(mensaje) => format!("404 Not Found: {}", mensaje),
            Self::InternalServerError(mensaje) => format!("500 Internal Server Error: {}", mensaje),
            Self::ValidationFailed(mensaje) => format!("422 Validation Failed: {}", mensaje),
            Self::Forbidden(mensaje) => format!("403 Forbidden: {}", mensaje),
            Self::BadRequest(mensaje) => format!("400 Bad Request: {}", mensaje),
        }
    }
}

impl ErrorHttp {
    pub fn obtener_estado(&self) -> EstadoHttp {
        match self {
            Self::NotFound(_) => EstadoHttp::NotFound,
            Self::InternalServerError(_) => EstadoHttp::InternalServerError,
            Self::ValidationFailed(_) => EstadoHttp::ValidationFailed,
            Self::Forbidden(_) => EstadoHttp::Forbidden,
            Self::BadRequest(_) => EstadoHttp::BadRequest,
        }
    }

    pub fn obtener_mensaje(&self) -> String {
        match self {
            Self::NotFound(mensaje) => mensaje.to_string(),
            Self::InternalServerError(mensaje) => mensaje.to_string(),
            Self::ValidationFailed(mensaje) => mensaje.to_string(),
            Self::Forbidden(mensaje) => mensaje.to_string(),
            Self::BadRequest(mensaje) => mensaje.to_string(),
        }
    }
}

use super::estado::Estado;

#[derive(Debug)]
pub enum Http {
    BadRequest(String),
    NotFound(String),
    InternalServerError(String),
}

impl Http {
    pub fn obtener_estado(&self) -> Estado {
        match self {
            Self::BadRequest(_) => Estado::BadRequest,
            Self::NotFound(_) => Estado::NotFound,
            Self::InternalServerError(_) => Estado::InternalServerError,
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

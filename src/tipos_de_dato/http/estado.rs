use std::fmt::Display;

pub enum Estado {
    Ok,
    NotFound,
    InternalServerError,
    BadRequest,
}

impl Estado {
    pub fn obtener_estado_y_mensaje(&self) -> (usize, String) {
        match self {
            Estado::Ok => (200, "OK".to_string()),
            Estado::NotFound => (404, "Not Found".to_string()),
            Estado::InternalServerError => (500, "Internal Server Error".to_string()),
            Estado::BadRequest => (400, "Bad Request".to_string()),
        }
    }
}

impl Display for Estado {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (estado, mensaje) = self.obtener_estado_y_mensaje();
        write!(f, "{} {}", estado, mensaje)
    }
}

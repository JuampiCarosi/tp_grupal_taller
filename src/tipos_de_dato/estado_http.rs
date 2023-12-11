use std::fmt::Display;

pub enum EstadoHttp {
    Ok,
    NotFound,
    InternalServerError,
    BadRequest,
}

impl EstadoHttp {
    pub fn obtener_estado_y_mensaje(&self) -> (usize, String) {
        match self {
            EstadoHttp::Ok => (200, "OK".to_string()),
            EstadoHttp::NotFound => (404, "Not Found".to_string()),
            EstadoHttp::InternalServerError => (500, "Internal Server Error".to_string()),
            EstadoHttp::BadRequest => (400, "Bad Request".to_string()),
        }
    }
}

impl Display for EstadoHttp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (estado, mensaje) = self.obtener_estado_y_mensaje();
        write!(f, "{} {}", estado, mensaje)
    }
}

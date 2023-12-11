use super::error::Http;

#[derive(Debug, PartialEq)]
pub enum Metodo {
    Get,
    Post,
    Put,
    Patch,
}

impl Metodo {
    pub fn from_string(metodo: &str) -> Result<Metodo, Http> {
        match metodo {
            "GET" => Ok(Metodo::Get),
            "POST" => Ok(Metodo::Post),
            "PUT" => Ok(Metodo::Put),
            "PATCH" => Ok(Metodo::Patch),
            _ => Err(Http::BadRequest(format!("Metodo {} no soportado", metodo))),
        }
    }
}

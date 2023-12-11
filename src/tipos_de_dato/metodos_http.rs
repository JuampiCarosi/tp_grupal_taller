use super::error_http::ErrorHttp;

#[derive(Debug)]
pub enum MetodoHttp {
    Get(String),
    Post(String),
    Put(String),
    Patch(String),
}

impl MetodoHttp {
    pub fn from_string(metodo: &str, ruta: &str) -> Result<MetodoHttp, ErrorHttp> {
        match metodo {
            "GET" => Ok(MetodoHttp::Get(ruta.to_string())),
            "POST" => Ok(MetodoHttp::Post(ruta.to_string())),
            "PUT" => Ok(MetodoHttp::Put(ruta.to_string())),
            "PATCH" => Ok(MetodoHttp::Patch(ruta.to_string())),
            _ => Err(ErrorHttp::BadRequest(format!(
                "Metodo {} no soportado",
                metodo
            ))),
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            MetodoHttp::Get(_) => "GET".to_string(),
            MetodoHttp::Post(_) => "POST".to_string(),
            MetodoHttp::Put(_) => "PUT".to_string(),
            MetodoHttp::Patch(_) => "PATCH".to_string(),
        }
    }
}

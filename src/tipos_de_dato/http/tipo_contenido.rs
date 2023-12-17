use std::collections::HashMap;

use super::error::ErrorHttp;

pub enum TipoContenido {
    Json,
    TextPlain,
}

impl TipoContenido {
    pub fn from_string(string: &str) -> Result<Self, String> {
        match string {
            "application/json" => Ok(Self::Json),
            _ => Err(format!("Tipo de contenido {} no soportado", string)),
        }
    }

    pub fn to_string(&self) -> Result<String, String> {
        match self {
            TipoContenido::Json => Ok("application/json".to_string()),
            TipoContenido::TextPlain => Ok("text/plain".to_string()),
        }
    }

    pub fn parsear_contenido(
        &self,
        contenido: &[u8],
    ) -> Result<HashMap<String, String>, ErrorHttp> {
        match self {
            Self::Json => {
                let comando: HashMap<String, String> = serde_json::from_slice(contenido)
                    .map_err(|e| ErrorHttp::BadRequest(e.to_string()))?;
                Ok(comando)
            }
            Self::TextPlain => Err(ErrorHttp::InternalServerError(format!(
                "No se puede parsear Texto planoen conido"
            ))),
        }
    }
}

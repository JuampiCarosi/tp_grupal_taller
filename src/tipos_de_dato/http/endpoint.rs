use std::{collections::HashMap, sync::Arc};

use crate::tipos_de_dato::logger::Logger;

use super::{error::ErrorHttp, metodos::MetodoHttp, request::Request, response::Response};

pub struct Endpoint {
    pub metodo: MetodoHttp,
    pub patron: String,
    pub handler: fn(Request, HashMap<String, String>, Arc<Logger>) -> Result<Response, ErrorHttp>,
}

impl Endpoint {
    pub fn new(
        metodo: MetodoHttp,
        patron: String,
        handler: fn(Request, HashMap<String, String>, Arc<Logger>) -> Result<Response, ErrorHttp>,
    ) -> Self {
        Self {
            metodo,
            patron: patron,
            handler,
        }
    }

    pub fn matchea_con_patron(&self, ruta: &str) -> Option<HashMap<String, String>> {
        let ruta_endpoint = self.patron.split("/").collect::<Vec<&str>>();
        let ruta_request = ruta.split("/").collect::<Vec<&str>>();

        if ruta_endpoint.len() != ruta_request.len() {
            return None;
        }

        let ultimo_ruta_request = match ruta_request.last() {
            Some(ruta) => ruta,
            None => return None,
        };
        if ultimo_ruta_request.is_empty() {
            return None;
        }

        let mut params = HashMap::new();

        for (ruta_endpoint, ruta_request) in ruta_endpoint.iter().zip(ruta_request.iter()) {
            if ruta_endpoint.starts_with("{") && ruta_endpoint.ends_with("}") {
                let key = ruta_endpoint[1..ruta_endpoint.len() - 1].to_string();
                params.insert(key, ruta_request.to_string());
                continue;
            }

            if ruta_endpoint != ruta_request {
                return None;
            }
        }

        Some(params)
    }
}

#[cfg(test)]

mod tests {
    use crate::tipos_de_dato::http::estado::EstadoHttp;

    use super::*;

    #[test]
    fn extraer_parametros_de_ruta_con_un_param() {
        let endpoint = Endpoint::new(
            MetodoHttp::Get,
            "/repos/{repo}/pulls".to_string(),
            |_, _, _| {
                Ok(Response::new(
                    Arc::new(Logger::new(std::path::PathBuf::from("server_logger.txt")).unwrap()),
                    EstadoHttp::Ok,
                    None,
                ))
            },
        );

        let params = endpoint.matchea_con_patron("/repos/messi/pulls").unwrap();
        assert_eq!(params.get("repo").unwrap(), "messi");

        let params = endpoint.matchea_con_patron("/repos/messi/");
        assert!(params.is_none());

        let params = endpoint.matchea_con_patron("/typo/messi/pulls");
        assert!(params.is_none());

        let params = endpoint.matchea_con_patron("/repos/messi/typo");
        assert!(params.is_none());
    }

    #[test]
    fn extraer_parametros_de_ruta_con_dos_param() {
        let endpoint = Endpoint::new(
            MetodoHttp::Get,
            "/repos/{repo}/pulls/{pull}".to_string(),
            |_, _, _| {
                Ok(Response::new(
                    Arc::new(Logger::new(std::path::PathBuf::from("server_logger.txt")).unwrap()),
                    EstadoHttp::Ok,
                    None,
                ))
            },
        );

        let params = endpoint.matchea_con_patron("/repos/messi/pulls/1").unwrap();
        assert_eq!(params.get("repo").unwrap(), "messi");
        assert_eq!(params.get("pull").unwrap(), "1");

        let params = endpoint.matchea_con_patron("/repos/messi/pulls/");
        assert!(params.is_none());

        let params = endpoint.matchea_con_patron("/typo/messi/pulls/1");
        assert!(params.is_none());

        let params = endpoint.matchea_con_patron("/repos/messi/typo/1");
        assert!(params.is_none());
    }
}

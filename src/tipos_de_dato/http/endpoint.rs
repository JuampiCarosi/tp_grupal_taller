use std::{collections::HashMap, sync::Arc};

use crate::tipos_de_dato::logger::Logger;

use super::{error::Http, metodos::Metodo, request::Request, response::Response};

pub struct Endpoint {
    pub metodo: Metodo,
    pub path: String,
    pub handler: fn(Request, HashMap<String, String>, Arc<Logger>) -> Result<Response, Http>,
}

impl Endpoint {
    pub fn new(
        metodo: Metodo,
        path: String,
        handler: fn(Request, HashMap<String, String>, Arc<Logger>) -> Result<Response, Http>,
    ) -> Self {
        Self {
            metodo,
            path,
            handler,
        }
    }

    pub fn extraer_parametros_de_ruta(&self, ruta: &str) -> Option<HashMap<String, String>> {
        let ruta_endpoint = self.path.split("/").collect::<Vec<&str>>();
        let ruta_request = ruta.split("/").collect::<Vec<&str>>();

        if ruta_endpoint.len() != ruta_request.len() {
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

// fn obtener_pull_request(
//     req: HttpRequest,
//     params: HashMap<String, String>,
//     logger: Arc<Logger>,
// ) -> Result<HttpResponse, ErrorHttp> {
//     let repo = params.get("repo").ok_or(ErrorHttp::InternalServerError(
//         "No se encontro el parametro repo".to_string(),
//     ))?;

//     // ...

//     Ok(HttpResponse::new(logger, EstadoHttp::Ok, None))
// }

// fn crear_pull_request(
//     req: Request,
//     params: HashMap<String, String>,
//     logger: Arc<Logger>,
// ) -> Result<Response, Error> {
//     let repo = params.get("repo").ok_or(Error::InternalServerError(
//         "No se encontro el parametro repo".to_string(),
//     ))?;

//     // ...

//     Ok(Response::new(logger, Estado::Ok, None))
// }

// fn agregar_a_router(rutas: &mut Vec<Endpoint>) {
//     let ruta_obtener_pr = Endpoint::new(
//         Metodo::Get,
//         "/repos/{repo}/pulls".to_string(),
//         obtener_pull_request,
//     );
//     rutas.push(ruta_obtener_pr);
// }

// pub fn main() {
//     let logger = Arc::new(Logger::new(PathBuf::from("server_logger.txt")).unwrap());
//     let req = Request {
//         metodo: Metodo::Get,
//         ruta: "/repos/messi/pulls".to_string(),
//         version: "HTTP/1.1".to_string(),
//         headers: HashMap::new(),
//         body: None,
//         logger: logger.clone(),
//     };

// let ruta_crear_pr = Endpoint::new(
//     Metodo::Post,
//     "/repos/{repo}/pulls".to_string(),
//     crear_pull_request,
// );

// let rutas = vec![ruta_obtener_pr, ruta_crear_pr];
// let mut endpoints: Vec<Endpoint> = Vec::new();

// let endpoint = endpoints
//     .iter()
//     .find(|endpoint| endpoint.metodo == req.metodo && ruta.es_ruta_correcta(&req.ruta));

//     for endpoint in endpoints {
//         if endpoint.metodo != req.metodo {
//             continue;
//         }

//         let params = match endpoint.extraer_parametros_de_ruta(&req.ruta) {
//             Some(params) => params,
//             None => continue,
//         };

//         let response = (endpoint.handler)(req, params, logger.clone()).unwrap();
//         response.enviar(stream).unwrap();
//         return;
//     }

//     let response = Response::new(logger, Estado::NotFound, None);
//     response.enviar(stream).unwrap();
// }

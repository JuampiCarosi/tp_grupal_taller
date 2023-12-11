use std::{collections::HashMap, path::PathBuf, sync::Arc};

use super::{
    error_http::ErrorHttp, estado_http::EstadoHttp, http_request::HttpRequest,
    http_response::HttpResponse, logger::Logger, metodos_http::MetodoHttp,
};

pub struct Endpoint {
    pub metodo: MetodoHttp,
    pub path: String,
    pub handler:
        fn(HttpRequest, HashMap<String, String>, Arc<Logger>) -> Result<HttpResponse, ErrorHttp>,
}

impl Endpoint {
    pub fn new(
        metodo: MetodoHttp,
        path: String,
        handler: fn(
            HttpRequest,
            HashMap<String, String>,
            Arc<Logger>,
        ) -> Result<HttpResponse, ErrorHttp>,
    ) -> Self {
        Self {
            metodo,
            path,
            handler,
        }
    }

    pub fn es_ruta_correcta(&self, ruta: &str) -> bool {
        let ruta_endpoint = self.path.split("/").collect::<Vec<&str>>();
        let ruta_request = ruta.split("/").collect::<Vec<&str>>();

        if ruta_endpoint.len() != ruta_request.len() {
            return false;
        }

        for (ruta_endpoint, ruta_request) in ruta_endpoint.iter().zip(ruta_request.iter()) {
            if ruta_endpoint.starts_with("{") && ruta_endpoint.ends_with("}") {
                continue;
            }

            if ruta_endpoint != ruta_request {
                return false;
            }
        }

        true
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

// fn agregar_a_router(rutas: &mut Vec<Endpoint>) {
//     let ruta_obtener_pr = Endpoint::new(
//         MetodoHttp::Get,
//         "/repos/{repo}/pulls".to_string(),
//         obtener_pull_request,
//     );
//     rutas.push(ruta_obtener_pr);
// }

// pub fn main() {
//     let logger = Arc::new(Logger::new(PathBuf::from("server_logger.txt")).unwrap());
//     let req = HttpRequest {
//         metodo: MetodoHttp::Get,
//         ruta: "/repos/messi/pulls".to_string(),
//         version: "HTTP/1.1".to_string(),
//         headers: HashMap::new(),
//         body: None,
//         logger: logger.clone(),
//     };

//     let ruta_crear_pr = Endpoint::new(
//         MetodoHttp::Post,
//         "/repos/{repo}/pulls".to_string(),
//         crear_pull_request,
//     );

//     let rutas = vec![ruta_obtener_pr, ruta_crear_pr];

//     let ruta = rutas
//         .iter()
//         .find(|ruta| ruta.metodo == req.metodo && ruta.es_ruta_correcta(&req.ruta));

//     let ruta = match ruta {
//         Some(ruta) => ruta,
//         None => {
//             // let response = HttpResponse::new(
//             //     req.logger.clone(),
//             //     EstadoHttp::NotFound,
//             //     Some("No se encontro la ruta"),
//             // );
//             // response.enviar(&mut ).unwrap();
//             return;
//         }
//     };

//     (ruta.handler)(req, HashMap::new(), logger.clone()).unwrap();
// }

use std::{collections::HashMap, sync::Arc};

use crate::tipos_de_dato::{
    http::{
        endpoint::Endpoint, error::ErrorHttp, estado::EstadoHttp, metodos::MetodoHttp,
        request::Request, response::Response,
    },
    logger::Logger,
};

use super::obtener_pull_request::{self, obtener_pull_request_de_params};

pub fn agregar_a_router(rutas: &mut Vec<Endpoint>) {
    let endpoint = Endpoint::new(
        MetodoHttp::Post,
        "/repos/{owner}/{repo}/pulls/{pull_number}/merge".to_string(),
        mergear_pull_request,
    );
    rutas.push(endpoint)
}

fn mergear_pull_request(
    request: Request,
    params: HashMap<String, String>,
    logger: Arc<Logger>,
) -> Result<Response, ErrorHttp> {
    let body = match request.body {
        Some(body) => body,
        None => {
            // por ahora dejo esto, pero desps vemos que hacer con los params del body
            return Err(ErrorHttp::BadRequest(
                "No se ha encontrado el cuerpo de la solicitud".to_string(),
            ));
        }
    };

    let dir_pull_request = obtener_pull_request::obtener_dir_pull_request(&params)?;
    let mut pull_request = obtener_pull_request_de_params(params)?;

    let rama_base = pull_request.rama_base.clone();
    let rama_head = pull_request.rama_head.clone();

    //logica para mergear

    pull_request.estado = "Cerrado".to_string();
    pull_request.guardar_pr(&dir_pull_request)?;
    let response = Response::new(logger, EstadoHttp::Ok, None);
    Ok(response)
}

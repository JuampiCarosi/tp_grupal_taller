use std::{collections::HashMap, sync::Arc};

use crate::tipos_de_dato::{
    http::{
        endpoint::Endpoint, error::ErrorHttp, estado::EstadoHttp, metodos::MetodoHttp,
        request::Request, response::Response,
    },
    logger::Logger,
};

use super::obtener_pull_request::obtener_pull_request_de_params;

pub fn agregar_a_router(rutas: &mut Vec<Endpoint>) {
    let endpoint = Endpoint::new(
        MetodoHttp::Get,
        "/repos/{repo}/pulls/{pull_number}/commits".to_string(),
        obtener_commits_pull_request,
    );
    rutas.push(endpoint)
}

fn obtener_commits_pull_request(
    request: Request,
    params: HashMap<String, String>,
    logger: Arc<Logger>,
) -> Result<Response, ErrorHttp> {
    let pull_request_commits = obtener_pull_request_de_params(params)?.commits;

    if pull_request_commits.is_empty() {
        //evaluar que hacer en este caso
    }

    let body_response = serde_json::to_string(&pull_request_commits).map_err(|e| {
        ErrorHttp::InternalServerError(format!("No se ha podido serializar el pull request: {}", e))
    })?;

    let response = Response::new(logger, EstadoHttp::Created, Some(&body_response));
    Ok(response)
}

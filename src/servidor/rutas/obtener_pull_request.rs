use std::{collections::HashMap, path::PathBuf, sync::Arc};

use crate::{
    servidor::pull_request::PullRequest,
    tipos_de_dato::{
        http::{
            endpoint::Endpoint, error::ErrorHttp, estado::EstadoHttp, metodos::MetodoHttp,
            request::Request, response::Response,
        },
        logger::Logger,
    },
};

pub fn agregar_a_router(rutas: &mut Vec<Endpoint>) {
    let endpoint = Endpoint::new(
        MetodoHttp::Get,
        "/repos/{repo}/pulls/{pull_number}".to_string(),
        obtener_pull_request,
    );
    rutas.push(endpoint)
}

fn obtener_pull_request(
    request: Request,
    params: HashMap<String, String>,
    logger: Arc<Logger>,
) -> Result<Response, ErrorHttp> {
    let dir_pull_request = obtener_dir_pull_request(&params)?;
    let pull_request = PullRequest::cargar_pr(&dir_pull_request)?;
    let body_response = serde_json::to_string(&pull_request).map_err(|e| {
        ErrorHttp::InternalServerError(format!("No se ha podido serializar el pull request: {}", e))
    })?;

    let response = Response::new(logger, EstadoHttp::Created, Some(&body_response));
    Ok(response)
}

fn obtener_dir_pull_request(params: &HashMap<String, String>) -> Result<PathBuf, ErrorHttp> {
    let repo = params.get("repo").ok_or_else(|| {
        ErrorHttp::InternalServerError("No se ha encontrado el nombre del repositorio".to_string())
    })?;
    let pull_number = params.get("pull_number").ok_or_else(|| {
        ErrorHttp::InternalServerError("No se ha encontrado el nombre del repositorio".to_string())
    })?;
    let dir_pull_request = PathBuf::from(format!("./srv/{repo}/pulls/{pull_number}"));

    if dir_pull_request.exists() {
        Ok(dir_pull_request)
    } else {
        Err(ErrorHttp::NotFound(format!(
            "No se encontro en el server {:?}",
            dir_pull_request
        )))
    }
}

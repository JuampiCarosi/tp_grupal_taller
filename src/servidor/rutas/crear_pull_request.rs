use std::{collections::HashMap, sync::Arc};

use crate::tipos_de_dato::{
    http::{
        endpoint::Endpoint, error::ErrorHttp, estado::EstadoHttp, metodos::MetodoHttp,
        request::Request, response::Response,
    },
    logger::Logger,
};

pub fn agregar_a_router(rutas: &mut Vec<Endpoint>) {
    let endpoint = Endpoint::new(
        MetodoHttp::Post,
        "/repos/{repo}/pulls".to_string(),
        crear_pull_request,
    );
    rutas.push(endpoint)
}

fn crear_pull_request(
    request: Request,
    params: HashMap<String, String>,
    logger: Arc<Logger>,
) -> Result<Response, ErrorHttp> {
    let repo = params.get("repo").unwrap();
    let body = request.body.clone().unwrap();

    let mut pull_request = HashMap::new();

    pull_request.insert("repo".to_string(), repo.to_string());

    let response = Response::new(
        logger,
        EstadoHttp::Created,
        Some(&serde_json::to_string(&pull_request).unwrap()),
    );
    Ok(response)
}

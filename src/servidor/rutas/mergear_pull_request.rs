use std::{collections::HashMap, path::PathBuf, sync::Arc};

use crate::{
    servidor::pull_request::PullRequest,
    tipos_de_dato::{
        comando::Ejecutar,
        comandos::merge::Merge,
        http::{
            endpoint::Endpoint, error::ErrorHttp, estado::EstadoHttp, metodos::MetodoHttp,
            request::Request, response::Response,
        },
        logger::Logger,
    },
    utils::{index, ramas},
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

fn armar_body_merge(hash_merge: String) -> String {
    let body_merge = format!(
        r#"{{
            "sha": "{}",
            "merged": true,
            "message": "Pull Request mergeado con exito"
        }}"#,
        hash_merge
    );
    body_merge
}

fn merge_ejecutado_con_exito(
    rama_base: &str,
    pull_request: &mut PullRequest,
    logger: Arc<Logger>,
    dir_pull_request: &PathBuf,
) -> Result<Response, ErrorHttp> {
    let hash_merge = ramas::obtener_hash_commit_asociado_rama(&rama_base).map_err(|e| {
        ErrorHttp::InternalServerError(format!(
            "No se ha podido obtener el hash del commit de la rama {}: {}",
            rama_base, e
        ))
    })?;
    let body_merge = armar_body_merge(hash_merge);
    pull_request.estado = "Cerrado".to_string();
    pull_request.guardar_pr(&dir_pull_request)?;
    let response = Response::new(logger, EstadoHttp::Ok, Some(&body_merge));
    Ok(response)
}

fn merge_fallo(logger: Arc<Logger>, error: String) -> Result<Response, ErrorHttp> {
    if index::hay_archivos_con_conflictos(logger.clone()) {
        let response = Response::new(logger, EstadoHttp::MergeNotAllowed, None);
        Ok(response)
    } else {
        Err(ErrorHttp::InternalServerError(format!(
            "No se ha podido mergear el pull request: {}",
            error
        )))
    }
}

fn mergear_pull_request(
    _request: Request,
    params: HashMap<String, String>,
    logger: Arc<Logger>,
) -> Result<Response, ErrorHttp> {
    // let body = match request.body {
    //     Some(body) => body,
    //     None => {
    //         // por ahora dejo esto, pero desps vemos que hacer con los params del body
    //         return Err(ErrorHttp::BadRequest(
    //             "No se ha encontrado el cuerpo de la solicitud".to_string(),
    //         ));
    //     }
    // };

    let dir_pull_request = obtener_pull_request::obtener_dir_pull_request(&params)?;
    let mut pull_request = obtener_pull_request_de_params(params)?;

    let rama_base = pull_request.rama_base.clone();
    let rama_head = pull_request.rama_head.clone();

    let mut merge = Merge {
        logger: logger.clone(),
        branch_actual: rama_base.clone(),
        branch_a_mergear: rama_head,
    };

    match merge.ejecutar() {
        Ok(_) => {
            merge_ejecutado_con_exito(&rama_base, &mut pull_request, logger, &dir_pull_request)
        }
        // esto falta terminarlo
        Err(e) => merge_fallo(logger, e.to_string()),
    }
}

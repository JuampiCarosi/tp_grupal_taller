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
    utils::{index, io, ramas},
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

fn verificar_sha_head(sha: &str, rama_head: &str) -> Result<bool, ErrorHttp> {
    let hash_head_previo_merge =
        ramas::obtener_hash_commit_asociado_rama(&rama_head).map_err(|error| {
            ErrorHttp::InternalServerError(format!(
                "No se ha podido obtener el hash del commit de la rama {}: {}",
                rama_head, error
            ))
        })?;
    Ok(sha == hash_head_previo_merge)
}

fn obtener_params_body(
    request: Request,
    rama_base: &str,
) -> Result<(bool, Option<&str>), ErrorHttp> {
    let body = match request.body {
        Some(body) => body,
        None => return Ok((false, None)),
    };

    if let Some(sha) = body.get("sha") {
        if !verificar_sha_head(sha, rama_base)? {
            return Ok((false, None));
        }
    }

    if let Some(merge_method) = body.get("merge_method") {
        match merge_method.as_str() {
            "squash" | "merge" => return Ok((true, Some("merge"))),
            "rebase" => return Ok((true, Some("rebase"))),
            _ => return Ok((false, None)),
        }
    };
    return Ok((true, Some("merge")));
}

fn merge_ejecutado_con_exito(
    rama_base: &str,
    pull_request: &mut PullRequest,
    logger: Arc<Logger>,
    dir_pull_request: &PathBuf,
) -> Result<Response, ErrorHttp> {
    let hash_merge = ramas::obtener_hash_commit_asociado_rama(&rama_base).map_err(|error| {
        ErrorHttp::InternalServerError(format!(
            "No se ha podido obtener el hash del commit de la rama {}: {}",
            rama_base, error
        ))
    })?;
    let body_merge = armar_body_merge(hash_merge);
    pull_request.estado = "closed".to_string();
    pull_request.guardar_pr(&dir_pull_request)?;
    let response = Response::new(logger, EstadoHttp::Ok, Some(&body_merge));
    Ok(response)
}

fn volver_a_estado_previo_al_merge() -> Result<(), ErrorHttp> {
    io::rm_directorio(".gir/MERGE_HEAD").map_err(|e| {
        ErrorHttp::InternalServerError(format!(
            "No se ha podido eliminar el archivo MERGE_HEAD: {}",
            e
        ))
    })?;
    index::limpiar_archivo_index().map_err(|error| {
        ErrorHttp::InternalServerError(format!(
            "No se ha podido limpiar el archivo index: {}",
            error
        ))
    })?;
    Ok(())
}

fn merge_ejecutado_con_fallos(logger: Arc<Logger>, error: String) -> Result<Response, ErrorHttp> {
    volver_a_estado_previo_al_merge()?;

    if index::hay_archivos_con_conflictos(logger.clone()) {
        let response = Response::new(logger, EstadoHttp::MethodNotAllowed, None);
        Ok(response)
    } else {
        Err(ErrorHttp::InternalServerError(format!(
            "No se ha podido mergear el pull request: {}",
            error
        )))
    }
}

fn mergear_pull_request_utilizando_merge(
    pull_request: &mut PullRequest,
    logger: Arc<Logger>,
    dir_pull_request: &PathBuf,
) -> Result<Response, ErrorHttp> {
    let rama_base = pull_request.rama_base.clone();
    let rama_head = pull_request.rama_head.clone();

    let mut merge = Merge {
        logger: logger.clone(),
        branch_actual: rama_base.clone(),
        branch_a_mergear: rama_head,
        abort: false,
    };

    match merge.ejecutar() {
        Ok(_) => merge_ejecutado_con_exito(&rama_base, pull_request, logger, &dir_pull_request),
        Err(error) => merge_ejecutado_con_fallos(logger, error.to_string()),
    }
}

fn mergear_pull_request_utilizando_rebase() -> Result<Response, ErrorHttp> {
    unimplemented!()
}

fn mergear_pull_request(
    request: Request,
    params: HashMap<String, String>,
    logger: Arc<Logger>,
) -> Result<Response, ErrorHttp> {
    let dir_pull_request = obtener_pull_request::obtener_dir_pull_request(&params)?;
    let mut pull_request = obtener_pull_request_de_params(&params)?;

    if pull_request.estado != "open" {
        let response = Response::new(logger, EstadoHttp::ValidationFailed, None);
        return Ok(response);
    }

    let (es_posible_mergear, merge_method) = obtener_params_body(request, &pull_request.rama_base)?;

    if !es_posible_mergear {
        let response = Response::new(logger, EstadoHttp::Conflict, None);
        return Ok(response);
    }

    match merge_method {
        Some("merge") | Some("squash") => {
            mergear_pull_request_utilizando_merge(&mut pull_request, logger, &dir_pull_request)
        }
        Some("rebase") => mergear_pull_request_utilizando_rebase(),
        _ => Err(ErrorHttp::InternalServerError(
            "No se ha podido mergear el pull request".to_string(),
        )),
    }
}

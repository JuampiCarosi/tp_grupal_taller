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
    utils,
};

pub fn agregar_a_router(rutas: &mut Vec<Endpoint>) {
    let endpoint = Endpoint::new(
        MetodoHttp::Get,
        "/repos/{repo}/pulls".to_string(),
        listar_pull_request,
    );
    rutas.push(endpoint)
}

fn listar_pull_request(
    _request: Request,
    params: HashMap<String, String>,
    logger: Arc<Logger>,
) -> Result<Response, ErrorHttp> {
    let lista_pull_request = obtener_pull_request_del_repositorio(params)?;

    if lista_pull_request.is_empty() {
        //evaluar que hacer en este caso: error o o no mandar nada?
    }

    let body_respuesta = serde_json::to_string(&lista_pull_request).map_err(|e| {
        ErrorHttp::InternalServerError(format!(
            "No se ha podido serializar la lista de pull request: {}",
            e
        ))
    })?;

    let respuesta = Response::new(logger, EstadoHttp::Ok, Some(&body_respuesta));
    Ok(respuesta)
}

fn obtener_pull_request_del_repositorio(
    params: HashMap<String, String>,
) -> Result<Vec<PullRequest>, ErrorHttp> {
    let dir_repositorio = obtener_y_verificar_repositorio_de_los_parametros(&params)?;
    let iterador_repo_dir = utils::io::leer_directorio(&dir_repositorio).map_err(|e| {
        ErrorHttp::InternalServerError(format!(
            "Fallo al leer el directorio {:?}: {e}",
            dir_repositorio
        ))
    })?;

    let mut lista_pull_request = Vec::new();
    for entrada_repo_dir in iterador_repo_dir {
        match entrada_repo_dir {
            Ok(archivo_pull_request) => {
                let pull_request = PullRequest::cargar_pr(&archivo_pull_request.path())?;
                lista_pull_request.push(pull_request);
            }
            Err(e) => {
                return Err(ErrorHttp::InternalServerError(format!(
                    "Fallo al leer los conteidoso del directorio {:?}: {e}",
                    dir_repositorio
                )));
            }
        }
    }

    Ok(lista_pull_request)
}

///Verifica que exista el repositorio en el server. En caso que se validen estas condicion,
/// se devuelve dir repositorio y caso contrario  error.
///
/// ## Resultado
/// - dir del repo recibo(EJ: `./srv/{repositorio}/pulls`)
fn obtener_y_verificar_repositorio_de_los_parametros(
    params: &HashMap<String, String>,
) -> Result<PathBuf, ErrorHttp> {
    let repo = params.get("repo").ok_or_else(|| {
        ErrorHttp::InternalServerError("No se ha encontrado el nombre del repositorio".to_string())
    })?;

    let dir_repositorio = PathBuf::from(format!("./srv/{repo}/pulls"));

    if dir_repositorio.exists() {
        Ok(dir_repositorio)
    } else {
        Err(ErrorHttp::ValidationFailed(format!(
            "No existe en el server el repositorio {repo}"
        )))
    }
}

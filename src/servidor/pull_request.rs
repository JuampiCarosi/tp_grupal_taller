use crate::{
    tipos_de_dato::{
        comandos::{log::Log, merge::Merge},
        http::error::ErrorHttp,
        logger::Logger,
        objetos::commit::CommitObj,
    },
    utils::{self, io},
};

use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf, sync::Arc};

#[derive(Serialize, Deserialize)]
pub struct PullRequest {
    pub numero: u64,
    #[serde(
        skip_serializing_if = "Option::is_none",
        default = "default_valor_opcional"
    )]
    pub titulo: Option<String>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        default = "default_valor_opcional"
    )]
    pub descripcion: Option<String>,
    ///representa el estado del pr: solo puede ser `open` o `close`
    pub estado: String,
    pub autor: String,
    pub rama_head: String,
    pub rama_base: String,
    pub fecha_creacion: String,
    pub fecha_modificacion: String,
    pub commits: Vec<CommitObj>,
}

fn default_valor_opcional() -> Option<String> {
    None
}

impl PullRequest {
    pub fn new(
        numero: u64,
        titulo: Option<String>,
        descripcion: Option<String>,
        estado: String,
        autor: String,
        rama_head: String,
        rama_base: String,
        fecha_creacion: String,
        fecha_modificacion: String,
        commits: Vec<CommitObj>,
    ) -> Self {
        Self {
            numero,
            titulo,
            descripcion,
            estado,
            rama_head,
            rama_base,
            fecha_creacion,
            fecha_modificacion,
            commits,
            autor,
        }
    }

    pub fn crear_pr(
        repositorio: &str,
        body: HashMap<String, String>,
        logger: Arc<Logger>,
    ) -> Result<PullRequest, ErrorHttp> {
        Self::verificar_repositorio(repositorio)?;

        let numero = Self::obtener_numero(repositorio)?;
        let titulo = Self::obtener_titulo(&body);
        let descripcion = Self::obtener_descripcion(&body);
        let estado = "open".to_string();
        let (autor, rama_head) = Self::obtener_autor_y_rama_head(repositorio, &body)?;
        let rama_base = Self::obtener_rama_base(repositorio, &body)?;
        let fecha_actual = Self::obtener_fecha_actual();
        let commits = Self::obtener_commits(repositorio, &rama_base, &rama_head, logger);
        // let commits = Self::obtener_commits(repositorio, &rama_head, &rama_head, logger)
        //     .map_err(|e| ErrorHttp::InternalServerError(e))?;

        Ok(PullRequest {
            numero,
            titulo,
            descripcion,
            estado,
            autor,
            rama_head,
            rama_base,
            fecha_creacion: fecha_actual.clone(),
            fecha_modificacion: fecha_actual,
            commits,
        })
    }

    fn obtener_fecha_actual() -> String {
        //despues completar
        "fecha actual".to_string()
    }

    fn verificar_repositorio(repositorio: &str) -> Result<(), ErrorHttp> {
        let dir_repositorio = PathBuf::from(format!("./srv/{repositorio}"));

        if dir_repositorio.exists() {
            Ok(())
        } else {
            Err(ErrorHttp::ValidationFailed(format!(
                "No existe en el server el repositorio {repositorio}"
            )))
        }
    }
    fn obtener_commits(
        repositorio: &str,
        rama_base: &str,
        rama_head: &str,
        logger: Arc<Logger>,
    ) -> Vec<CommitObj> {
        //utils::io::cambiar_directorio(format!("srv/{repositorio}"))?;
        // let merge = Merge {
        //     logger: logger.clone(),
        //     branch_actual: rama_base.to_string(),
        //     branch_a_mergear: rama_head.to_string(),
        // };
        // let hash_ultimo_commit = Merge::obtener_commit_de_branch(rama_head)?;
        // let ultimo_commit = CommitObj::from_hash(hash_ultimo_commit, logger.clone())?;
        // let commits = Log::obtener_listas_de_commits(ultimo_commit, logger.clone())?;
        // let hash_commit_base = merge.obtener_commit_base_entre_dos_branches()?;
        // utils::io::cambiar_directorio(format!("../../"))?;

        // let commits_spliteados: Vec<&[CommitObj]> = commits
        //     .split(|commit| commit.hash == hash_commit_base)
        //     .collect();

        // commits_spliteados
        //     .get(0)
        //     .ok_or("No se encontro el commit base".to_string())
        //     .map(|commits| commits.to_vec())

        Vec::new()
    }

    fn obtener_numero(repositorio: &str) -> Result<u64, ErrorHttp> {
        let direccion = PathBuf::from(format!("./srv/{repositorio}/pulls"));
        if !direccion.exists() {
            return Ok(0);
        }

        utils::io::cantidad_entradas_dir(&direccion).map_err(|_| {
            ErrorHttp::InternalServerError("Fallo al obtener el numero del pr".to_string())
        })
    }

    fn obtener_rama_base(
        repositorio: &str,
        body: &HashMap<String, String>,
    ) -> Result<String, ErrorHttp> {
        if let Some(rama_base) = body.get("base") {
            Self::validar_rama(rama_base, repositorio)?;
            Ok(rama_base.to_string())
        } else {
            Err(ErrorHttp::ValidationFailed(
                "Falta el parametro 'head' en el body de la request".to_string(),
            ))
        }
    }
    fn obtener_autor_y_rama_head(
        repositorio: &str,
        body: &HashMap<String, String>,
    ) -> Result<(String, String), ErrorHttp> {
        if let Some(autor_y_rama_head) = body.get("head") {
            let (autor, rama_head) = Self::separara_autor_y_rama_head(autor_y_rama_head)?;
            Self::validar_rama(&rama_head, repositorio)?;
            Ok((autor, rama_head))
        } else {
            Err(ErrorHttp::ValidationFailed(
                "Falta el parametro 'head' en el body de la request".to_string(),
            ))
        }
    }
    //Comprueba si existe en
    fn validar_rama(rama: &str, repositorio: &str) -> Result<(), ErrorHttp> {
        let direccion = PathBuf::from(format!("./srv/{repositorio}/refs/heads/{rama}"));

        if !direccion.exists() {
            Err(ErrorHttp::ValidationFailed(format!(
                "No existe la rama {rama} en el repositorio {repositorio}"
            )))
        } else {
            Ok(())
        }
    }

    fn separara_autor_y_rama_head(autor_y_rama_head: &str) -> Result<(String, String), ErrorHttp> {
        if let Some((autor, rama_head)) = autor_y_rama_head.split_once(':') {
            Ok((autor.to_string(), rama_head.to_string()))
        } else {
            Err(ErrorHttp::ValidationFailed(format!(
                "Fallo al separar el autor de rama head: {autor_y_rama_head}"
            )))
        }
    }

    fn obtener_titulo(body: &HashMap<String, String>) -> Option<String> {
        if let Some(titulo) = body.get("title") {
            Some(titulo.to_owned())
        } else {
            None
        }
    }

    fn obtener_descripcion(body: &HashMap<String, String>) -> Option<String> {
        if let Some(descripcion) = body.get("body") {
            Some(descripcion.to_owned())
        } else {
            None
        }
    }

    pub fn guardar_pr(&self, direccion: PathBuf) -> Result<(), ErrorHttp> {
        let pr_serializado = serde_json::to_string(&self).map_err(|e| {
            ErrorHttp::InternalServerError(format!(
                "No se ha podido serializar el pull request: {}",
                e
            ))
        })?;
        io::escribir_bytes(direccion, pr_serializado.as_bytes()).map_err(|e| {
            ErrorHttp::InternalServerError(format!(
                "No se ha podido guardar el pull request: {}",
                e
            ))
        })?;
        Ok(())
    }

    pub fn cargar_pr(direccion: &str) -> Result<PullRequest, String> {
        let contenido = io::leer_a_string(direccion)?;
        let pr: PullRequest = serde_json::from_str(&contenido).unwrap();
        Ok(pr)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::fs::remove_file;

    #[test]
    fn test01_guardar_pr() {
        let pr = PullRequest::new(
            1,
            Option::Some(String::from("Titulazo")),
            Option::Some(String::from("Descripcion")),
            String::from("open"),
            String::from("Autor"),
            String::from("Rama head"),
            String::from("Rama base"),
            String::from("Fecha creacion"),
            String::from("Fecha modificacion"),
            Vec::new(),
        );
        pr.guardar_pr(PathBuf::from("test_dir/test01.json"))
            .unwrap();
        let pr_cargado = PullRequest::cargar_pr("test_dir/test01.json").unwrap();
        assert_eq!(pr.numero, pr_cargado.numero);
        assert_eq!(pr.titulo, pr_cargado.titulo);
        assert_eq!(pr.descripcion, pr_cargado.descripcion);
        assert_eq!(pr.fecha_creacion, pr_cargado.fecha_creacion);
        assert_eq!(pr.fecha_modificacion, pr_cargado.fecha_modificacion);
        assert_eq!(pr.commits.len(), pr_cargado.commits.len());
        remove_file("test_dir/test02.json").unwrap();
    }

    #[test]
    fn test02_se_puede_guardar_y_cargar_un_pr_con_un_campo_que_no_se_seriliza() {
        let pr = PullRequest::new(
            1,
            None,
            None,
            String::from("open"),
            String::from("Autor"),
            String::from("Rama head"),
            String::from("Rama base"),
            String::from("Fecha creacion"),
            String::from("Fecha modificacion"),
            Vec::new(),
        );
        pr.guardar_pr(PathBuf::from("test_dir/test02.json"))
            .unwrap();
        let pr_cargado = PullRequest::cargar_pr("test_dir/test02.json").unwrap();
        assert_eq!(pr.numero, pr_cargado.numero);
        assert_eq!(pr.titulo, pr_cargado.titulo);
        assert_eq!(pr.descripcion, pr_cargado.descripcion);
        assert_eq!(pr.fecha_creacion, pr_cargado.fecha_creacion);
        assert_eq!(pr.fecha_modificacion, pr_cargado.fecha_modificacion);
        assert_eq!(pr.commits.len(), pr_cargado.commits.len());
        remove_file("test_dir/test02.json").unwrap();
    }
}

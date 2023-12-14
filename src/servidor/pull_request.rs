use crate::{
    tipos_de_dato::{
        comandos::{log::Log, merge::Merge},
        http::error::ErrorHttp,
        logger::Logger,
        objetos::commit::CommitObj,
    },
    utils::{self, io},
};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf, sync::Arc};

const OPEN: &str = "open";
const CLOSE: &str = "close";

#[derive(Serialize, Deserialize, Debug)]
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
            autor,
        }
    }

    pub fn crear_pr(
        repositorio: &str,
        body: HashMap<String, String>,
    ) -> Result<PullRequest, ErrorHttp> {
        Self::verificar_repositorio(repositorio)?;

        let numero = Self::obtener_numero(repositorio)?;
        let titulo = Self::obtener_titulo(&body);
        let descripcion = Self::obtener_descripcion(&body);
        let estado = OPEN.to_string();
        let (autor, rama_head) = Self::obtener_autor_y_rama_head(repositorio, &body)?;
        let rama_base = Self::obtener_rama_base(repositorio, &body)?;
        let fecha_actual = Self::obtener_fecha_actual();

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
        })
    }

    pub fn actualizar(
        &mut self,
        body: HashMap<String, String>,
        repositorio: &str,
    ) -> Result<bool, ErrorHttp> {
        if self.estado == CLOSE.to_string() {
            return Ok(false);
        }

        let se_actualizo_titulo = self.actualizar_titulo(&body);
        let se_actualizo_descripcion = self.actualizar_descripcion(&body);
        let se_actulizo_estado = self.actulizar_estado(&body)?;
        let se_actualiza_rama_base = self.actualizar_rama_base(&body, repositorio)?;

        let se_actualizo_el_pull_request = se_actualiza_rama_base
            || se_actualizo_descripcion
            || se_actulizo_estado
            || se_actualizo_titulo;

        if se_actualizo_el_pull_request {
            self.fecha_modificacion = Self::obtener_fecha_actual();
        }

        Ok(se_actualizo_el_pull_request)
    }

    fn actualizar_rama_base(
        &mut self,
        body: &HashMap<String, String>,
        repositorio: &str,
    ) -> Result<bool, ErrorHttp> {
        if let Some(nueva_rama_base) = body.get("base") {
            Self::validar_rama(nueva_rama_base, repositorio)?;
            Self::verificar_rama_base_distinta_de_head(&self.rama_head, nueva_rama_base)?;

            let se_actualizo_rama_base = self.rama_base != *nueva_rama_base;
            self.rama_base = nueva_rama_base.to_owned();
            Ok(se_actualizo_rama_base)
        } else {
            Ok(false)
        }
    }

    fn verificar_rama_base_distinta_de_head(
        rama_head: &str,
        rama_base: &str,
    ) -> Result<(), ErrorHttp> {
        if *rama_base == *rama_head {
            return Err(ErrorHttp::ValidationFailed(format!(
                "Rama base ({rama_base}) y rama head ({rama_head}) no puede ser iguales"
            )));
        }

        Ok(())
    }

    fn actulizar_estado(&mut self, body: &HashMap<String, String>) -> Result<bool, ErrorHttp> {
        if let Some(estado) = body.get("status") {
            if estado != OPEN && estado != CLOSE {
                return Err(ErrorHttp::ValidationFailed(format!(
                    "El status {estado} no coincide con ninguno de los posibles: `open` o `close`"
                )));
            }

            let se_cambio_estado = self.estado != *estado;
            self.estado = estado.to_owned();
            Ok(se_cambio_estado)
        } else {
            Ok(false)
        }
    }

    fn actualizar_descripcion(&mut self, body: &HashMap<String, String>) -> bool {
        let descripcion_nueva = Self::obtener_descripcion(body);

        let se_actualizo_descripcion = self.descripcion != descripcion_nueva;

        self.descripcion = descripcion_nueva;
        se_actualizo_descripcion
    }

    fn actualizar_titulo(&mut self, body: &HashMap<String, String>) -> bool {
        let titulo_nuevo = Self::obtener_titulo(body);

        let se_actualizo_titulo = self.titulo != titulo_nuevo;

        self.titulo = titulo_nuevo;
        se_actualizo_titulo
    }

    fn obtener_fecha_actual() -> String {
        let ahora: DateTime<Utc> = Utc::now();
        ahora.to_rfc3339()
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

    pub fn obtener_commits(
        &self,
        repositorio: &str,
        logger: Arc<Logger>,
    ) -> Result<Vec<CommitObj>, ErrorHttp> {
        self._obtener_commits(repositorio, logger)
            .map_err(|e| ErrorHttp::InternalServerError(e))
    }

    fn _obtener_commits(
        &self,
        repositorio: &str,
        logger: Arc<Logger>,
    ) -> Result<Vec<CommitObj>, String> {
        utils::io::cambiar_directorio(format!("srv/{repositorio}"))?;

        let hash_ultimo_commit = Merge::obtener_commit_de_branch(&self.rama_head)?;
        let ultimo_commit = CommitObj::from_hash(hash_ultimo_commit, logger.clone())?;
        let commits = Log::obtener_listas_de_commits(ultimo_commit, logger.clone())?;
        let hash_commit_base = Merge::obtener_commit_base_entre_dos_branches(
            &self.rama_base,
            &self.rama_head,
            logger.clone(),
        )?;
        utils::io::cambiar_directorio(format!("../../"))?;

        let commits_spliteados: Vec<&[CommitObj]> = commits
            .split(|commit| commit.hash == hash_commit_base)
            .collect();

        commits_spliteados
            .get(0)
            .ok_or("No se encontro el commit base".to_string())
            .map(|commits| commits.to_vec())
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
                "Falta el parametro 'base' en el body de la request".to_string(),
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
        let direccion = PathBuf::from(format!("./srv/{repositorio}/.gir/refs/heads/{rama}"));
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

    pub fn guardar_pr(&self, direccion: &PathBuf) -> Result<(), ErrorHttp> {
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

    pub fn cargar_pr(direccion: &PathBuf) -> Result<PullRequest, ErrorHttp> {
        let contenido_pull_request = utils::io::leer_a_string(direccion).map_err(|e| {
            ErrorHttp::InternalServerError(format!("Fallo al leer la entrada {:?}: {e}", direccion))
        })?;
        let pull_request =
            serde_json::from_str::<PullRequest>(&contenido_pull_request).map_err(|e| {
                ErrorHttp::InternalServerError(format!(
                    "Fallo al serializar el contenido {contenido_pull_request}: {e}"
                ))
            })?;
        Ok(pull_request)
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
        );
        let direccion = PathBuf::from("tmp/test01.json");
        pr.guardar_pr(&direccion).unwrap();
        let pr_cargado = PullRequest::cargar_pr(&direccion).unwrap();
        assert_eq!(pr.numero, pr_cargado.numero);
        assert_eq!(pr.titulo, pr_cargado.titulo);
        assert_eq!(pr.descripcion, pr_cargado.descripcion);
        assert_eq!(pr.fecha_creacion, pr_cargado.fecha_creacion);
        assert_eq!(pr.fecha_modificacion, pr_cargado.fecha_modificacion);
        remove_file("tmp/test01.json").unwrap();
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
        );
        let direccion = PathBuf::from("test_dir/test02.json");
        pr.guardar_pr(&direccion).unwrap();
        let pr_cargado = PullRequest::cargar_pr(&direccion).unwrap();
        assert_eq!(pr.numero, pr_cargado.numero);
        assert_eq!(pr.titulo, pr_cargado.titulo);
        assert_eq!(pr.descripcion, pr_cargado.descripcion);
        assert_eq!(pr.fecha_creacion, pr_cargado.fecha_creacion);
        assert_eq!(pr.fecha_modificacion, pr_cargado.fecha_modificacion);
        remove_file("test_dir/test02.json").unwrap();
    }

    #[test]
    fn test03_se_puede_actualizar_el_titulo() {
        let mut pr = PullRequest::new(
            1,
            None,
            None,
            String::from("open"),
            String::from("Autor"),
            String::from("Rama head"),
            String::from("Rama base"),
            String::from("Fecha creacion"),
            String::from("Fecha modificacion"),
        );

        let titulo_a_cambiar = "Si ves esto funciona".to_string();
        let mut body = HashMap::new();
        body.insert("title".to_string(), titulo_a_cambiar.clone());
        pr.actualizar_titulo(&body);

        assert_eq!(pr.titulo, Some(titulo_a_cambiar));
    }
}

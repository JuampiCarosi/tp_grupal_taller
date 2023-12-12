use crate::{tipos_de_dato::objetos::commit::CommitObj, utils::io};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf};

const TITULO_PORDEFECTO: String = "".to_string();
const DESCRIPCION_PORDEFECTO: String = "".to_string();

#[derive(Serialize, Deserialize)]
pub struct PullRequest {
    pub numero: u64,
    pub titulo: String, // es un campo opcional - pensar en el opcion
    pub descripcion: String,
    pub esta_abierto: bool,
    pub autor: String,
    pub rama_head: String,
    pub rama_base: String,
    pub fecha_creacion: String,
    pub fecha_modificacion: String,
    pub commits: Vec<CommitObj>,
}

impl PullRequest {
    pub fn new(
        numero: u64,
        titulo: String,
        descripcion: String,
        esta_abierto: bool,
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
            esta_abierto,
            autor,
            rama_head,
            rama_base,
            fecha_creacion,
            fecha_modificacion,
            commits,
        }
    }

    pub fn crear_pr(repositorio: &str, parametros: HashMap<String, String>) -> PullRequest {
        let titulo = Self::obtener_titulo(parametros);
        let descripcion = Self::obtener_descripcion(parametros);
        let (autor, rama_head) = Self::obtener_autor_y_rama_head(repositorio, parametros);
    }

    fn obtener_autor_y_rama_head(
        repositorio: &str,
        parametros: HashMap<String, String>,
    ) -> (String, String) {
        let direccion = PathBuf::from(format!("./srv/{repositorio}/refs/heads"));
    }
    fn obtener_titulo(parametros: HashMap<String, String>) -> String {
        if let Some(titulo) = parametros.get("title") {
            titulo.to_owned()
        } else {
            TITULO_PORDEFECTO
        }
    }

    fn obtener_descripcion(parametros: HashMap<String, String>) -> String {
        if let Some(descripcion) = parametros.get("body") {
            descripcion.to_owned()
        } else {
            DESCRIPCION_PORDEFECTO
        }
    }

    pub fn guardar_pr(&self, direccion: PathBuf) -> Result<(), String> {
        let pr_serializado = serde_json::to_string(&self).map_err(|e| e.to_string())?;
        io::escribir_bytes(direccion, pr_serializado.as_bytes())?;
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
            String::from("Titulo"),
            String::from("Descripcion"),
            true,
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
        assert_eq!(pr.esta_abierto, pr_cargado.esta_abierto);
        assert_eq!(pr.autor, pr_cargado.autor);
        assert_eq!(pr.fecha_creacion, pr_cargado.fecha_creacion);
        assert_eq!(pr.fecha_modificacion, pr_cargado.fecha_modificacion);
        assert_eq!(pr.commits.len(), pr_cargado.commits.len());
        remove_file("test_dir/test01.json").unwrap();
    }
}

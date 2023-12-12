use std::{fs::File, io::{Write, Read}};
use crate::tipos_de_dato::objetos::commit::CommitObj;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct PullRequest {
    pub numero: u64,
    pub titulo: String,
    pub descripcion: String,
    pub esta_abierto: bool,
    pub autor: String, 
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
            fecha_creacion,
            fecha_modificacion,
            commits,
        }
    }

    pub fn guardar_pr(&self, direccion: &str) {
        let mut archivo = File::create(direccion).unwrap();
        let pr_serializado = serde_json::to_string(&self).unwrap();
        archivo.write_all(pr_serializado.as_bytes()).unwrap();
    }

    pub fn cargar_pr(direccion: &str) -> PullRequest {
        let mut archivo = File::open(direccion).unwrap();
        let mut contenido = String::new();
        archivo.read_to_string(&mut contenido).unwrap();
        let pr: PullRequest = serde_json::from_str(&contenido).unwrap();
        pr
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
            String::from("Fecha creacion"),
            String::from("Fecha modificacion"),
            Vec::new(),
        );
        pr.guardar_pr("test_dir/test01.json");
        let pr_cargado = PullRequest::cargar_pr("test_dir/test01.json");
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
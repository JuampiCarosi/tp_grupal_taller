use std::io::prelude::*;
use std::{fs::OpenOptions, path::PathBuf, sync::Arc};

use crate::utils::index::{self, escribir_index};
use crate::{
    tipos_de_dato::{
        comandos::write_tree::conseguir_arbol_from_hash_commit,
        logger::Logger,
        objetos::{commit::CommitObj, tree::Tree},
    },
    utils::io,
};

use super::checkout::Checkout;
use super::{commit::Commit, log::Log, merge::Merge};

pub struct Rebase {
    pub rama: Option<String>,
    pub logger: Arc<Logger>,
    pub abort: bool,
    pub continue_: bool,
}

impl Rebase {
    pub fn from(args: Vec<String>, logger: Arc<Logger>) -> Result<Rebase, String> {
        if args.len() != 1 {
            return Err("Se esperaba un argumento".to_string());
        }

        let arg = args.get(0).ok_or("Se esperaba un argumento")?;
        match arg.as_str() {
            "--abort" => Ok(Rebase {
                rama: None,
                logger,
                abort: true,
                continue_: false,
            }),
            "--continue" => Ok(Rebase {
                rama: None,
                logger,
                abort: false,
                continue_: true,
            }),
            _ => Ok(Rebase {
                rama: Some(arg.clone()),
                logger,
                abort: false,
                continue_: false,
            }),
        }
    }

    fn obtener_commit_base_entre_dos_branches(&self, rama: &str) -> Result<String, String> {
        let hash_commit_actual = Commit::obtener_hash_commit_actual()?;
        let hash_commit_a_rebasear = Merge::obtener_commit_de_branch(rama)?;

        let commit_obj_actual = CommitObj::from_hash(hash_commit_actual, self.logger.clone())?;
        let commit_obj_a_rebasear =
            CommitObj::from_hash(hash_commit_a_rebasear, self.logger.clone())?;

        let commits_branch_actual =
            Log::obtener_listas_de_commits(commit_obj_actual, self.logger.clone())?;
        let commits_branch_a_rebasear =
            Log::obtener_listas_de_commits(commit_obj_a_rebasear, self.logger.clone())?;

        for commit_actual in commits_branch_actual {
            for commit_branch_merge in commits_branch_a_rebasear.clone() {
                if commit_actual.hash == commit_branch_merge.hash {
                    return Ok(commit_actual.hash);
                }
            }
        }
        Err("No se encontro un commit base entre las dos ramas".to_string())
    }

    fn obtener_commits_a_aplicar(&self, rama: &str) -> Result<Vec<CommitObj>, String> {
        let hash_ultimo_commit = Commit::obtener_hash_commit_actual()?;
        let ultimo_commit = CommitObj::from_hash(hash_ultimo_commit, self.logger.clone())?;
        let commits = Log::obtener_listas_de_commits(ultimo_commit, self.logger.clone())?;
        let hash_commit_base = self.obtener_commit_base_entre_dos_branches(rama)?;
        let commis_spliteados: Vec<&[CommitObj]> = commits
            .split(|commit| commit.hash == hash_commit_base)
            .collect();

        commis_spliteados
            .get(0)
            .ok_or("No se encontro el commit base".to_string())
            .map(|commits| commits.to_vec())
    }

    fn crear_carpeta_rebase(
        &self,
        commits_a_aplicar: &[CommitObj],
        tip_nuevo: &str,
    ) -> Result<(), String> {
        io::crear_directorio(".gir/rebase-merge")?;
        io::escribir_bytes(".gir/rebase-merge/end", commits_a_aplicar.len().to_string())?;

        let mut archivo_to_do = OpenOptions::new()
            .write(true)
            .append(true)
            .create(true)
            .open(".gir/rebase-merge/git-rebase-todo")
            .map_err(|_| "No se pudo abrir el archivo .gir/rebase-merge/git-rebase-todo")?;

        for commit in commits_a_aplicar.iter() {
            writeln!(archivo_to_do, "pick {} {}", commit.hash, commit.mensaje).map_err(|_| {
                "No se pudo escribir en el archivo .gir/rebase-merge/git-rebase-todo"
            })?;
        }

        let ref_head = io::leer_a_string(".gir/HEAD")?;
        io::escribir_bytes(".gir/rebase-merge/head-name", &ref_head)?;

        let head = Commit::obtener_hash_commit_actual()?;
        io::escribir_bytes(".gir/rebase-merge/orig-head", head)?;
        io::escribir_bytes(".gir/rebase-merge/msgnum", 0.to_string())?;
        io::escribir_bytes(".gir/rebase-merge/onto", tip_nuevo)?;

        Ok(())
    }

    fn actualizar_carpeta_rebase(&self, commit: &CommitObj) -> Result<(), String> {
        let to_do = io::leer_a_string(".gir/rebase-merge/git-rebase-todo")?;
        let mut to_do = to_do.lines().collect::<Vec<&str>>();
        to_do.remove(0);
        let to_do = to_do.join("\n");
        io::escribir_bytes(".gir/rebase-merge/git-rebase-todo", to_do)?;

        let mut archivo_done = OpenOptions::new()
            .write(true)
            .append(true)
            .create(true)
            .open(".gir/rebase-merge/done")
            .map_err(|_| "No se pudo abrir el archivo .gir/rebase-merge/done")?;

        writeln!(archivo_done, "pick {} {}", commit.hash, commit.mensaje)
            .map_err(|_| "No se pudo escribir en el archivo .gir/rebase-merge/done")?;

        let msgnum = io::leer_a_string(".gir/rebase-merge/msgnum")?;
        let msgnum = msgnum
            .parse::<usize>()
            .map_err(|_| "No se pudo parsear msgnum")?;
        let msgnum = msgnum + 1;
        io::escribir_bytes(".gir/rebase-merge/msgnum", msgnum.to_string())?;
        io::escribir_bytes(".gir/rebase-merge/message", commit.mensaje.clone())?;

        Ok(())
    }

    fn actualizar_lista_de_commits_aplicados(&self, commit_sha: &str) -> Result<(), String> {
        let mut archivo = OpenOptions::new()
            .write(true)
            .append(true)
            .open(".gir/rebase-merge/rewritten-list")
            .map_err(|_| "No se pudo abrir el archivo .gir/rebase-merge/rewritten-list")?;

        let tip = Commit::obtener_hash_commit_actual()?;

        writeln!(archivo, "{} {}", tip, commit_sha)
            .map_err(|_| "No se pudo escribir en el archivo .gir/rebase-merge/rewritten-list")?;

        Ok(())
    }

    fn primera_vez(&self) -> Result<String, String> {
        let rama = self.rama.as_ref().ok_or("No se especifico una rama")?;

        self.logger.log("Rebaseando...");
        let commits_a_aplicar = self.obtener_commits_a_aplicar(&rama)?;

        let tip_nuevo = io::leer_a_string(format!(".gir/refs/heads/{}", rama))?;
        self.crear_carpeta_rebase(&commits_a_aplicar, &tip_nuevo)?;

        let branch_actual = Commit::obtener_branch_actual()?;
        io::escribir_bytes(format!(".gir/refs/heads/{branch_actual}"), &tip_nuevo)?;

        let hash_arbol_commit =
            conseguir_arbol_from_hash_commit(&tip_nuevo, ".gir/objects/".to_string());

        let arbol = Tree::from_hash(hash_arbol_commit, PathBuf::from("./"), self.logger.clone())?;

        arbol.escribir_en_directorio()?;

        self.rebasear_commits(commits_a_aplicar)?;
        self.logger.log("Rebase finalizado");

        Ok(format!(
            "Se aplicaron los commits de la rama {} a la rama {}",
            rama,
            Commit::obtener_branch_actual()?
        ))
    }

    fn rebasear_commits(&self, commits_a_aplicar: Vec<CommitObj>) -> Result<(), String> {
        for commit in commits_a_aplicar {
            self.actualizar_carpeta_rebase(&commit)?;

            let conflictos = commit.aplicar_a_directorio()?;
            if !conflictos.is_empty() {
                io::escribir_bytes(".gir/rebase-merge/stopped-sha", commit.hash)?;
                let mut index = index::leer_index(self.logger.clone())?;

                let mut index_nuevo: Vec<_> = index
                    .iter_mut()
                    .map(|objeto_index| {
                        if conflictos.contains(&objeto_index.objeto.obtener_path()) {
                            objeto_index.merge = true;
                        }
                        objeto_index.clone()
                    })
                    .collect();

                escribir_index(self.logger.clone(), &mut index_nuevo)?;

                return Err("Se encontro un conflicto".to_string());
            }

            self.actualizar_lista_de_commits_aplicados(&commit.hash)?;

            let comando_commit = Commit {
                mensaje: commit.mensaje,
                logger: self.logger.clone(),
            };
            comando_commit.ejecutar()?;
        }

        Ok(())
    }

    fn abortar(&self) -> Result<String, String> {
        let head_name = io::leer_a_string(".gir/rebase-merge/head-name")?;
        let orig_head = io::leer_a_string(".gir/rebase-merge/orig-head")?;

        let rama = head_name
            .split('/')
            .last()
            .ok_or("No se pudo obtener la rama")?;

        io::escribir_bytes(format!(".gir/refs/heads/{}", rama), &orig_head)?;

        let tree = Checkout::obtener_arbol_commit_actual(self.logger.clone())?;

        tree.escribir_en_directorio()?;

        io::rm_directorio(".gir/rebase-merge")?;

        index::limpiar_archivo_index()?;

        self.logger.log("Rebase abortado");
        Ok("Rebase abortado".to_string())
    }

    fn continuar(&self) -> Result<String, String> {
        let mensaje_commit = io::leer_a_string(".gir/rebase-merge/message")?;
        let commit = Commit::from(
            &mut vec!["-m".to_string(), mensaje_commit],
            self.logger.clone(),
        )?;
        commit.ejecutar()?;

        let contenido_to_do = io::leer_a_string(".gir/rebase-merge/git-rebase-todo")?;
        let lineas_to_do = contenido_to_do.lines().collect::<Vec<&str>>();

        let mut commits_restantes = Vec::new();

        for linea in lineas_to_do {
            let linea_spliteada = linea.split(' ').collect::<Vec<&str>>();
            if linea_spliteada.len() != 3 {
                return Err("No se pudo parsear la linea del archivo git-rebase-todo".to_string());
            }

            let commit = CommitObj::from_hash(linea_spliteada[1].to_string(), self.logger.clone())?;
            commits_restantes.push(commit);
        }

        self.rebasear_commits(commits_restantes)?;
        Ok("Rebase terminado con extito".to_string())
    }

    pub fn ejecutar(&self) -> Result<String, String> {
        if self.abort {
            return self.abortar();
        }

        if self.continue_ {
            return self.continuar();
        }

        if !self.rama.is_none() {
            return self.primera_vez();
        }

        Err("No se especifico una rama".to_string())
    }
}

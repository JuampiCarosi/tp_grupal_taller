use std::{path::PathBuf, sync::Arc};

use crate::{
    tipos_de_dato::{
        comandos::{
            status::{self, Status},
            write_tree::conseguir_arbol_from_hash_commit,
        },
        logger::Logger,
        objetos::{commit::CommitObj, tree::Tree},
    },
    utils::io::{escribir_bytes, leer_a_string, leer_bytes},
};

use super::{commit::Commit, log::Log, merge::Merge};

pub struct Rebase {
    pub rama: String,
    pub logger: Arc<Logger>,
}

impl Rebase {
    pub fn from(args: Vec<String>, logger: Arc<Logger>) -> Result<Rebase, String> {
        if args.len() != 1 {
            return Err("Se esperaba un argumento".to_string());
        }

        let rama = args[0].clone();

        Ok(Rebase { rama, logger })
    }

    fn obtener_commit_base_entre_dos_branches(&self) -> Result<String, String> {
        let hash_commit_actual = Commit::obtener_hash_commit_actual()?;
        let hash_commit_a_rebasear = Merge::obtener_commit_de_branch(&self.rama)?;

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

    fn obtener_commits_a_aplicar(&self) -> Result<Vec<CommitObj>, String> {
        let hash_ultimo_commit = Commit::obtener_hash_commit_actual()?;
        let ultimo_commit = CommitObj::from_hash(hash_ultimo_commit, self.logger.clone())?;
        let commits = Log::obtener_listas_de_commits(ultimo_commit, self.logger.clone())?;
        let hash_commit_base = self.obtener_commit_base_entre_dos_branches()?;
        let commis_spliteados: Vec<&[CommitObj]> = commits
            .split(|commit| commit.hash == hash_commit_base)
            .collect();

        commis_spliteados
            .get(0)
            .ok_or("No se encontro el commit base".to_string())
            .map(|commits| commits.to_vec())
    }

    pub fn ejecutar(&self) -> Result<String, String> {
        self.logger.log("Rebaseando...".to_string());
        let commits_a_aplicar = self.obtener_commits_a_aplicar()?;

        let head_nuevo = leer_a_string(format!(".gir/refs/heads/{}", self.rama))?; //
        let branch_actual = Commit::obtener_branch_actual()?;
        escribir_bytes(format!(".gir/refs/heads/{branch_actual}"), &head_nuevo)?;

        let hash_arbol_commit =
            conseguir_arbol_from_hash_commit(&head_nuevo, ".gir/objects/".to_string());

        let arbol = Tree::from_hash(hash_arbol_commit, PathBuf::from("./"), self.logger.clone())?;

        arbol.escribir_en_directorio()?;

        for commit in commits_a_aplicar {
            let mensaje = commit.mensaje.clone();
            commit.aplicar_a_directorio()?;
            let comando_commit = Commit {
                mensaje,
                logger: self.logger.clone(),
            };
            comando_commit.ejecutar()?;
        }
        self.logger.log("Rebase finalizado".to_string());

        Ok(format!(
            "Se aplicaron los commits de la rama {} a la rama {}",
            self.rama,
            Commit::obtener_branch_actual()?
        ))
    }
}

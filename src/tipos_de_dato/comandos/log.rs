use std::cmp::Reverse;
use std::collections::HashMap;
use std::sync::Arc;

use crate::tipos_de_dato::logger::Logger;

use crate::io;
use crate::tipos_de_dato::comandos::checkout::Checkout;
use crate::tipos_de_dato::objetos::commit::CommitObj;
use crate::utils::compresion::descomprimir_objeto;

use super::commit::Commit;

pub struct Log {
    branch: String,
    logger: Arc<Logger>,
}

impl Log {
    pub fn from(args: &mut Vec<String>, logger: Arc<Logger>) -> Result<Log, String> {
        if args.len() > 2 {
            return Err("Cantidad de argumentos invalida".to_string());
        }
        let branch = match args.pop() {
            Some(branch) => {
                let ramas_disponibles = Checkout::obtener_ramas()?;
                if ramas_disponibles.contains(&branch) {
                    branch
                } else {
                    return Err(format!("La rama {} no existe", branch));
                }
            }
            None => Commit::obtener_branch_actual()
                .map_err(|e| format!("No se pudo obtener la rama actual\n{}", e))?,
        };
        Ok(Log { branch, logger })
    }

    fn obtener_commit_branch(branch: &str) -> Result<String, String> {
        let hash_commit = io::leer_a_string(format!(".gir/refs/heads/{}", branch))?;
        Ok(hash_commit.to_string())
    }

    pub fn obtener_listas_de_commits(commit: CommitObj) -> Result<Vec<CommitObj>, String> {
        let mut commits: HashMap<String, CommitObj> = HashMap::new();
        let mut commits_a_revisar: Vec<CommitObj> = Vec::new();
        commits_a_revisar.push(commit);

        while !commits_a_revisar.is_empty() {
            let commit = commits_a_revisar.pop().unwrap();
            if commits.contains_key(&commit.hash) {
                break;
            }
            commits.insert(commit.hash.clone(), commit.clone());
            for padre in commit.padres {
                let commit_padre = CommitObj::from_hash(padre)?;
                commits_a_revisar.push(commit_padre);
            }
        }

        let mut commits_vec = Vec::from_iter(commits.values().cloned());
        commits_vec.sort_by_key(|commit| Reverse(commit.date.tiempo.clone()));

        Ok(commits_vec)
    }

    pub fn ejecutar(&self) -> Result<String, String> {
        self.logger.log("Ejecutando comando log".to_string());
        let hash_commit = Self::obtener_commit_branch(&self.branch)?;
        if hash_commit.is_empty() {
            return Err(format!("La rama {} no tiene commits", self.branch));
        }

        let objeto_commit = CommitObj::from_hash(hash_commit)?;

        let commits = Self::obtener_listas_de_commits(objeto_commit)?;

        let mut log = String::new();

        for commit in commits {
            log.push_str(&commit.format_log()?);
            log.push_str("\n");
        }

        Ok(log)
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn test01_creacion_de_log_sin_branch() {
        let mut args = vec![];
        let logger = Arc::new(Logger::new(PathBuf::from("tmp/log")).unwrap());
        let log = Log::from(&mut args, logger).unwrap();
        assert_eq!(log.branch, "master");
    }

    #[test]
    fn test02_creacion_de_log_indicando_branch() {
        io::escribir_bytes(".gir/refs/heads/rama", "hash".as_bytes()).unwrap();
        let mut args = vec!["rama".to_string()];
        let logger = Arc::new(Logger::new(PathBuf::from("tmp/log")).unwrap());
        let log = Log::from(&mut args, logger).unwrap();
        assert_eq!(log.branch, "rama");
        std::fs::remove_file(".gir/refs/heads/rama").unwrap();
    }

    #[test]
    #[should_panic(expected = "La rama rama no existe")]
    fn test03_error_al_usar_branch_inexistente() {
        let mut args = vec!["rama".to_string()];
        let logger = Arc::new(Logger::new(PathBuf::from("tmp/log")).unwrap());
        let _ = Log::from(&mut args, logger).unwrap();
    }
}

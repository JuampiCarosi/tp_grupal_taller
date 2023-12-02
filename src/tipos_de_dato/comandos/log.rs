use std::cmp::Reverse;
use std::collections::HashMap;
use std::sync::Arc;

use crate::tipos_de_dato::logger::Logger;

use crate::tipos_de_dato::comandos::checkout::Checkout;
use crate::tipos_de_dato::objetos::commit::CommitObj;
use crate::utils::io;

use super::commit::Commit;

pub struct Log {
    /// Rama de la cual se quiere obtener el log.
    branch: String,
    /// Logger para registrar los eventos ocurridos durante la ejecucion del comando.
    logger: Arc<Logger>,
}

impl Log {
    /// Crea un comando log a partir de los argumentos pasados por linea de comandos.
    /// Si no se especifica una rama, se usa la rama actual.
    /// En caso de tener argumentos invalidos devuelve error.
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

    /// Obtiene el hash del commit al que apunta la rama pasada por parametro.
    fn obtener_commit_branch(branch: &str) -> Result<String, String> {
        let hash_commit = io::leer_a_string(format!(".gir/refs/heads/{}", branch))?;
        Ok(hash_commit.to_string())
    }

    /// Obtiene todos los commits que son padres del commit pasado por parametro.
    /// Devuelve un vector con los commits ordenados por fecha.
    /// En caso de haber un commit repetido, solo se utiliza uno.
    pub fn obtener_listas_de_commits(
        commit: CommitObj,
        logger: Arc<Logger>,
    ) -> Result<Vec<CommitObj>, String> {
        let mut commits: HashMap<String, CommitObj> = HashMap::new();
        let mut commits_a_revisar: Vec<CommitObj> = Vec::new();
        commits_a_revisar.push(commit);

        while let Some(commit) = commits_a_revisar.pop() {
            if commits.contains_key(&commit.hash) {
                break;
            }
            commits.insert(commit.hash.clone(), commit.clone());
            for padre in commit.padres {
                let commit_padre = CommitObj::from_hash(padre, logger.clone())?;
                commits_a_revisar.push(commit_padre);
            }
        }

        let mut commits_vec = Vec::from_iter(commits.values().cloned());
        commits_vec.sort_by_key(|commit| Reverse(commit.date.tiempo.clone()));

        Ok(commits_vec)
    }

    /// Ejecuta el comando log.
    /// Devuelve un string con el log de los commits de la rama.
    /// En caso de no haber commits devuelve un mensaje y corta la ejecucion.
    pub fn ejecutar(&self) -> Result<String, String> {
        self.logger.log("Ejecutando comando log");
        let hash_commit = Self::obtener_commit_branch(&self.branch)?;
        if hash_commit.is_empty() {
            return Err(format!("La rama {} no tiene commits", self.branch));
        }

        let objeto_commit = CommitObj::from_hash(hash_commit, self.logger.clone())?;

        let commits = Self::obtener_listas_de_commits(objeto_commit, self.logger.clone())?;

        let mut log = String::new();

        for commit in commits {
            log.push_str(&commit.format_log()?);
            log.push('\n');
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
        let logger = Arc::new(Logger::new(PathBuf::from("tmp/log_test01")).unwrap());
        let log = Log::from(&mut args, logger).unwrap();
        assert_eq!(log.branch, "master");
    }

    #[test]
    fn test02_creacion_de_log_indicando_branch() {
        io::escribir_bytes(".gir/refs/heads/rama", "hash".as_bytes()).unwrap();
        let mut args = vec!["rama".to_string()];
        let logger = Arc::new(Logger::new(PathBuf::from("tmp/log_test02")).unwrap());
        let log = Log::from(&mut args, logger).unwrap();
        assert_eq!(log.branch, "rama");
    }

    #[test]
    fn test03_obtener_commit_branch() {
        io::escribir_bytes(".gir/refs/heads/rama", "hash".as_bytes()).unwrap();
        let hash = Log::obtener_commit_branch("rama").unwrap();
        assert_eq!(hash, "hash");
        std::fs::remove_file(".gir/refs/heads/rama").unwrap();
    }

    #[test]
    #[should_panic(expected = "La rama rama no existe")]
    fn test04_error_al_usar_branch_inexistente() {
        let mut args = vec!["rama".to_string()];
        let logger = Arc::new(Logger::new(PathBuf::from("tmp/log_test04")).unwrap());
        let _ = Log::from(&mut args, logger).unwrap();
    }
}

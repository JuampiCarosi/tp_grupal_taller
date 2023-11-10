use std::path::PathBuf;

use crate::utils::{self, io};

#[derive(Debug, Clone)]
pub struct RemoteInfo {
    pub nombre: String,
    pub url: String,
}

#[derive(Debug, Clone)]

pub struct RamasInfo {
    nombre: String,
    remote: String,
    merge: String,
}

pub struct Config {
    pub remotes: Vec<RemoteInfo>,
    pub ramas: Vec<RamasInfo>,
}

impl Config {
    pub fn leer_config() -> Result<Config, String> {
        let contenido = io::leer_a_string(".gir/config")?;
        let contenido_spliteado = contenido.split('[').collect::<Vec<&str>>();
        let mut remotes: Vec<RemoteInfo> = Vec::new();
        let mut ramas: Vec<RamasInfo> = Vec::new();

        if contenido.is_empty() {
            return Ok(Config { remotes, ramas });
        }

        for contenido_raw in contenido_spliteado {
            if contenido_raw.is_empty() {
                continue;
            }

            let contenido = contenido_raw.split(']').collect::<Vec<&str>>();
            let header = contenido[0].split_whitespace().collect::<Vec<&str>>();
            match header[0] {
                "remote" => {
                    let informacion_remote = contenido[1].split(" = ").collect::<Vec<&str>>();

                    if informacion_remote[0].trim() != "url" {
                        return Err("Error en el archivo de configuracion".to_string());
                    }

                    let remote = RemoteInfo {
                        nombre: header[1].replace('\"', "").to_string(),
                        url: informacion_remote[1].trim().to_string(),
                    };
                    remotes.push(remote);
                }
                "branch" => {
                    let informacion_branch = contenido[1].split('\n').collect::<Vec<&str>>();
                    let mut remote = String::new();
                    let mut merge = String::new();

                    for linea in informacion_branch {
                        let linea = linea.split(" = ").collect::<Vec<&str>>();
                        match linea[0] {
                            "remote" => remote = linea[1].to_string(),
                            "merge" => merge = linea[1].to_string(),
                            _ => return Err("Error en el archivo de configuracion".to_string()),
                        }
                    }

                    if remote.is_empty() || merge.is_empty() {
                        return Err("Error en el archivo de configuracion".to_string());
                    }

                    let branch = RamasInfo {
                        nombre: header[1].replace('\"', "").to_string(),
                        remote,
                        merge,
                    };

                    ramas.push(branch);
                }
                _ => return Err("Error en el archivo de configuracion".to_string()),
            }
        }

        Ok(Config { remotes, ramas })
    }

    ///busca dentro de los remote del config, si remote efectivente existe.
    /// Si existe devuelve true, caso contrario false
    pub fn existe_remote(&self, remote: &String) -> bool {
        self.remotes.iter().any(|x| x.nombre == *remote)
    }

    ///en caso de existir un remoto asosiado a la rama actual, lo devuelve
    pub fn obtener_remoto_rama_actual(&self) -> Option<String> {
        let rama_actual = utils::ramas::obtener_rama_actual().err()?;

        match self.ramas.iter().find(|&rama| rama.nombre == rama_actual) {
            Some(rama) => Some((*rama.remote).to_string()),
            None => None,
        }
    }

    pub fn guardar_config(&self) -> Result<(), String> {
        let mut contenido = String::new();

        for remote in &self.remotes {
            contenido.push_str(&format!("[remote \"{}\"]\n", remote.nombre));
            contenido.push_str(&format!("   url = {}\n", remote.url));
        }

        for branch in &self.ramas {
            contenido.push_str(&format!("[branch \"{}\"]\n", branch.nombre));
            contenido.push_str(&format!("   remote = {}\n", branch.remote));
            contenido.push_str(&format!("   merge = {}\n", branch.merge));
        }

        io::escribir_bytes(PathBuf::from(".gir/config"), contenido)?;

        Ok(())
    }
}

#[cfg(test)]

mod tests {

    use super::*;

    #[test]
    fn test01_guardar_config() {
        let remote = RemoteInfo {
            nombre: "origin".to_string(),
            url: "localhost:3000".to_string(),
        };

        let config = Config {
            remotes: vec![remote],
            ramas: vec![],
        };

        config.guardar_config().unwrap();

        let file = io::leer_a_string(".gir/config").unwrap();

        assert_eq!(file, "[remote \"origin\"]\n   url = localhost:3000\n");
    }

    #[test]

    fn test02_leer_config() {
        let remote = RemoteInfo {
            nombre: "origin".to_string(),
            url: "localhost:3000".to_string(),
        };

        let config = Config {
            remotes: vec![remote],
            ramas: vec![],
        };

        config.guardar_config().unwrap();

        let config = Config::leer_config().unwrap();

        assert_eq!(config.remotes[0].nombre, "origin");
        assert_eq!(config.remotes[0].url, "localhost:3000");
    }

    #[test]
    fn test03_existe_remoto() {
        let mut config = Config {
            remotes: vec![],
            ramas: vec![],
        };

        //caso en el que config vacio, devulve false
        assert!(!config.existe_remote(&"origin".to_string()));

        let remote = RemoteInfo {
            nombre: "config".to_string(),
            url: "localhost:3000".to_string(),
        };

        config.remotes.push(remote);

        //coso tiene algo pero no lo que se busca
        assert!(!config.existe_remote(&"origin".to_string()));

        let remote = RemoteInfo {
            nombre: "origin".to_string(),
            url: "localhost:3000".to_string(),
        };

        config.remotes.push(remote);
        assert!(config.existe_remote(&"origin".to_string()));
    }
}

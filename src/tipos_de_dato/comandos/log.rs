use std::rc::Rc;

use chrono::{FixedOffset, LocalResult, TimeZone};

use crate::tipos_de_dato::logger::Logger;

use crate::tipos_de_dato::comandos::checkout::Checkout;
use crate::{io, utilidades_de_compresion};

use super::commit::Commit;

pub struct Log {
    branch: String,
    logger: Rc<Logger>,
}

fn timestamp_archivo_log(
    timestamp: i64,
    offset_horas: i32,
    offset_minutos: i32,
) -> Result<String, String> {
    let offset_seconds = offset_horas * 3600 + offset_minutos * 60;
    let offset_str = format!(
        "{:>+03}{:02}",
        offset_seconds / 3600,
        (offset_seconds % 3600) / 60
    );

    let offset = match FixedOffset::east_opt(offset_seconds) {
        Some(offset) => offset,
        None => return Err("No se pudo obtener el offset".to_string()),
    };

    let datetime = match offset.timestamp_opt(timestamp, 0) {
        LocalResult::Single(datetime) => datetime,
        _ => return Err("No se pudo obtener el datetime".to_string()),
    };

    let datetime_formateado = datetime.format("%a %b %d %H:%M:%S %Y");
    let formatted_timestamp = format!("{} {}", datetime_formateado, offset_str);

    Ok(formatted_timestamp)
}

impl Log {
    pub fn from(args: &mut Vec<String>, logger: Rc<Logger>) -> Result<Log, String> {
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

    pub fn conseguir_padre_desde_contenido_commit(contenido: &str) -> String {
        let contenido_spliteado = contenido.split('\n').collect::<Vec<&str>>();
        let siguiente_padre = contenido_spliteado[1].split(' ').collect::<Vec<&str>>()[1];
        siguiente_padre.to_string()
    }

    fn calcular_date_desde_linea(tiempo: &str, offset: &str) -> Result<String, String> {
        let timestamp = match tiempo.parse::<i64>() {
            Ok(timestamp) => timestamp,
            Err(_) => return Err("No se pudo obtener el timestamp".to_string()),
        };
        let (horas, minutos) = offset.split_at(3);
        let offset_horas = horas[0..3].parse::<i32>().unwrap_or_else(|_| -3);
        let offset_minutos = minutos.parse::<i32>().unwrap_or_else(|_| 0);
        Ok(timestamp_archivo_log(
            timestamp,
            offset_horas,
            offset_minutos,
        )?)
    }

    fn armar_contenido_log(
        contenido: &str,
        branch_actual: &str,
        hash_commit: String,
    ) -> Result<String, String> {
        let contenido_splitteado_del_header = contenido.split('\0').collect::<Vec<&str>>();
        let lineas_contenido = contenido_splitteado_del_header[1]
            .split('\n')
            .collect::<Vec<&str>>();
        let linea_autor_splitteada = lineas_contenido[2].split(' ').collect::<Vec<&str>>();
        let nombre_autor = linea_autor_splitteada[1];
        let mail_autor = linea_autor_splitteada[2];
        let date =
            Self::calcular_date_desde_linea(linea_autor_splitteada[3], linea_autor_splitteada[4])?;
        let mensaje = lineas_contenido[5];
        let mut branch_format = format!("({})", branch_actual);
        if branch_actual == "" {
            branch_format = "".to_string();
        }
        Ok(format!(
            "commit {} {}\nAutor: {} {}\nDate: {}\n\n     {}\n",
            hash_commit, branch_format, nombre_autor, mail_autor, date, mensaje
        ))
    }

    pub fn ejecutar(&self) -> Result<String, String> {
        self.logger.log("Ejecutando comando log".to_string());
        let mut hash_commit = Self::obtener_commit_branch(&self.branch)?;
        if hash_commit.is_empty() {
            return Err(format!("La rama {} no tiene commits", self.branch));
        }
        let mut i = 1;
        loop {
            let contenido = utilidades_de_compresion::descomprimir_objeto(hash_commit.clone())?;
            let siguiente_padre = Self::conseguir_padre_desde_contenido_commit(&contenido);
            let contenido_a_mostrar = match i {
                1 => {
                    i -= 1;
                    Self::armar_contenido_log(&contenido, &self.branch, hash_commit.clone())?
                }
                _ => Self::armar_contenido_log(&contenido, "", hash_commit.clone())?,
            };
            println!("{}", contenido_a_mostrar);
            if siguiente_padre.is_empty() {
                break;
            }
            hash_commit = siguiente_padre.to_string();
        }
        Ok("Log terminado".to_string())
    }
}

#[cfg(test)]
mod test {
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn test01_creacion_de_log_sin_branch() {
        let mut args = vec![];
        let logger = Rc::new(Logger::new(PathBuf::from("tmp/log")).unwrap());
        let log = Log::from(&mut args, logger).unwrap();
        assert_eq!(log.branch, "master");
    }

    #[test]
    fn test02_creacion_de_log_indicando_branch() {
        io::escribir_bytes(".gir/refs/heads/rama", "hash".as_bytes()).unwrap();
        let mut args = vec!["rama".to_string()];
        let logger = Rc::new(Logger::new(PathBuf::from("tmp/log")).unwrap());
        let log = Log::from(&mut args, logger).unwrap();
        assert_eq!(log.branch, "rama");
        std::fs::remove_file(".gir/refs/heads/rama").unwrap();
    }

    #[test]
    #[should_panic(expected = "La rama rama no existe")]
    fn test03_error_al_usar_branch_inexistente() {
        let mut args = vec!["rama".to_string()];
        let logger = Rc::new(Logger::new(PathBuf::from("tmp/log")).unwrap());
        let _ = Log::from(&mut args, logger).unwrap();
    }

    #[test]
    fn test04_rearmar_timestamp_log() {
        let timestamp = 1234567890;
        let offset_horas = -03;
        let offset_minutos = 00;
        let timestamp_formateado =
            timestamp_archivo_log(timestamp, offset_horas, offset_minutos).unwrap();
        let timestamp_formateado_esperado = "Fri Feb 13 20:31:30 2009 -0300";
        assert_eq!(timestamp_formateado, timestamp_formateado_esperado);
    }

    #[test]
    fn test05_armar_contenido_log() {
        let contenido = "commit 142\0tree 1a2b3c4d5e6f7g8h9i0j1k2l3m4n5o6p7q8r9s0t\nparent 1a2b3c4d5e6f7g8h9i0j1k2l3m4n5o6p7q8r9s0t\nauthor nombre_apellido <mail> 1234567890 -0300\ncommitter nombre_apellido <mail> 1234567890 -0300\n\nMensaje del commit";
        let branch_actual = "master";
        let hash_commit = "1a2b3c4d5e6f7g8h9i0j1k2l3m4n5o6p7q8r9s0t".to_string();
        let contenido_log =
            Log::armar_contenido_log(contenido, branch_actual, hash_commit).unwrap();
        let contenido_log_esperado = "commit 1a2b3c4d5e6f7g8h9i0j1k2l3m4n5o6p7q8r9s0t (master)\nAutor: nombre_apellido <mail>\nDate: Fri Feb 13 20:31:30 2009 -0300\n\n     Mensaje del commit\n";
        assert_eq!(contenido_log, contenido_log_esperado);
    }
}

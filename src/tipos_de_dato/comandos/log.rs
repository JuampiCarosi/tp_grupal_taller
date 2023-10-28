use std::rc::Rc;

use chrono::{FixedOffset, TimeZone, LocalResult};

use crate::tipos_de_dato::logger::Logger;

use crate::tipos_de_dato::comandos::checkout::Checkout;
use crate::{utilidades_de_compresion, io};

use super::commit::Commit;


pub struct Log {
    branch: String,
    logger: Rc<Logger>
}

fn timestamp_archivo_log(timestamp: i64, offset_horas: i32, offset_minutos: i32) -> Result<String, String> {

    let offset = match FixedOffset::east_opt(offset_horas * 3600 + offset_minutos * 60) {
        Some(offset) => offset,
        None => return Err("No se pudo obtener el offset".to_string()),
    };
    let datetime = match offset.timestamp_opt(timestamp, 0) {
        LocalResult::Single(datetime) => datetime,
        _ => return Err("No se pudo obtener el datetime".to_string()),
    };
    let datetime_formateado = datetime.format("%a %b %d %H:%M:%S %Y %:z,");
    Ok(datetime_formateado.to_string())
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
            },
            None => Commit::obtener_branch_actual().map_err(|e| format!("No se pudo obtener la rama actual\n{}", e))?,
        };
        Ok(Log { branch, logger })
    }

    fn obtener_commit_branch(branch: &str) -> Result<String, String> {
        let hash_commit = io::leer_a_string(format!(".gir/refs/heads/{}", branch))?;
        Ok(hash_commit.to_string())
    }

    fn conseguir_padre_desde_contenido_commit(contenido: &str) -> String {
        let contenido_spliteado = contenido.split('\n').collect::<Vec<&str>>();
        let siguiente_padre = contenido_spliteado[1].split(' ').collect::<Vec<&str>>()[1];
        siguiente_padre.to_string()
    }

    fn armar_contenido_log(contenido: &str, branch_actual: &str, hash_commit: String) -> Result<String, String> {
        let contenido_splitteado_del_header = contenido.split('\0').collect::<Vec<&str>>();
        let lineas_contenido = contenido_splitteado_del_header[1].split('\n').collect::<Vec<&str>>();
        let nombre_autor = lineas_contenido[2].split(' ').collect::<Vec<&str>>()[1];
        let mail_autor = lineas_contenido[2].split(' ').collect::<Vec<&str>>()[2];
        let timestamp = match lineas_contenido[2].split(' ').collect::<Vec<&str>>()[3].parse::<i64>() {
            Ok(timestamp) => timestamp,
            Err(_) => return Err("No se pudo obtener el timestamp".to_string()),
        };
        let date = timestamp_archivo_log(timestamp, -03, 00)?;
        Ok(format!("commit {} ({})\nAutor: {} {}\nDate: {}\n", hash_commit, branch_actual, nombre_autor, mail_autor, date))
    }

    pub fn ejecutar(&self) -> Result<String, String>{
        self.logger.log("Ejecutando comando log".to_string());
        let mut hash_commit = Self::obtener_commit_branch(&self.branch)?;
        if hash_commit.is_empty() {
            return Err(format!("La rama {} no tiene commits", self.branch));
        }
        loop {
            let contenido = utilidades_de_compresion::descomprimir_objeto(hash_commit.clone())?;
            let siguiente_padre = Self::conseguir_padre_desde_contenido_commit(&contenido);
            let contenido_a_mostrar = Self::armar_contenido_log(&contenido, &self.branch, hash_commit)?;
            println!("{}", contenido_a_mostrar);
            if siguiente_padre.is_empty() {
                break;
            }
            hash_commit = siguiente_padre.to_string();
        }
        Ok("Log terminado".to_string())
    }
}
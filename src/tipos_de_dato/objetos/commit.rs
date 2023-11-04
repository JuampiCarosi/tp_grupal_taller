use chrono::{FixedOffset, LocalResult, TimeZone};

use crate::tipos_de_dato::comandos::cat_file;

#[derive(Clone)]
pub struct CommitObj {
    pub hash: String,
    pub hash_tree: String,
    pub autor: String,
    pub mail: String,
    pub date: Date,
    pub mensaje: String,
    pub padres: Vec<String>,
}
#[derive(Clone, Debug)]

pub struct Date {
    pub tiempo: String,
    pub offset: String,
}

impl CommitObj {
    fn format_timestamp(
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

    fn formatear_date(date: &Date) -> Result<String, String> {
        let timestamp = match date.tiempo.parse::<i64>() {
            Ok(timestamp) => timestamp,
            Err(_) => return Err("No se pudo obtener el timestamp".to_string()),
        };
        let (horas, minutos) = date.offset.split_at(3);
        let offset_horas = horas[0..3].parse::<i32>().unwrap_or_else(|_| -3);
        let offset_minutos = minutos.parse::<i32>().unwrap_or_else(|_| 0);
        Ok(Self::format_timestamp(
            timestamp,
            offset_horas,
            offset_minutos,
        )?)
    }

    pub fn from_hash(hash: String) -> Result<CommitObj, String> {
        if hash.len() != 40 {
            return Err("Hash invalido".to_string());
        }
        let (_header, contenido) = cat_file::obtener_contenido_objeto(hash.clone())?;
        let mut padres: Vec<String> = Vec::new();
        let mut autor_option: Option<String> = None;
        let mut mail_option: Option<String> = None;
        let mut date_option: Option<Date> = None;
        let mut hash_tree_option: Option<String> = None;

        for linea in contenido.split("\n") {
            let linea_splitteada = linea.split(' ').collect::<Vec<&str>>();
            match linea_splitteada[0] {
                "parent" => padres.push(linea_splitteada[1].to_string()),
                "author" => {
                    autor_option = Some(linea_splitteada[1].to_string());
                    mail_option = Some(linea_splitteada[2].to_string());
                    date_option = Some(Date {
                        tiempo: linea_splitteada[3].to_string(),
                        offset: linea_splitteada[4].to_string(),
                    });
                }
                "tree" => {
                    hash_tree_option = Some(linea_splitteada[1].to_string());
                }
                "commiter" => {}
                _ => break,
            }
        }

        let (autor, mail, date, hash_tree) =
            match (autor_option, mail_option, date_option, hash_tree_option) {
                (Some(autor), Some(mail), Some(date), Some(hash_tree)) => {
                    (autor, mail, date, hash_tree)
                }
                _ => return Err("No se pudo obtener el contenido del commit".to_string()),
            };

        let linea_splitteada_contenido = contenido.splitn(2, "\n\n").collect::<Vec<&str>>();
        let mensaje = linea_splitteada_contenido[1].to_string();

        let objeto = CommitObj {
            hash,
            hash_tree,
            autor,
            mail,
            date,
            mensaje,
            padres,
        };

        Ok(objeto)
    }

    pub fn format_log(&self) -> Result<String, String> {
        let color_amarillo = "\x1B[33m";
        let color_reset = "\x1B[0m";
        let mut log = format!("{}commit {} {}\n", color_amarillo, self.hash, color_reset);

        if self.padres.len() > 1 {
            log.push_str("Merge: ");
            for padre in &self.padres {
                log.push_str(&format!("{} ", &padre[..7]));
            }
            log.push('\n');
        }
        log.push_str(&format!("Autor: {} {}\n", self.autor, self.mail));
        log.push_str(&format!("Date: {}\n", Self::formatear_date(&self.date)?));
        log.push_str(&format!("\n     {}\n", self.mensaje));
        Ok(log)
    }
}

#[cfg(test)]

mod test {

    use super::{CommitObj, Date};

    #[test]
    fn test01_rearmar_timestamp_log() {
        let timestamp = 1234567890;
        let offset_horas = -03;
        let offset_minutos = 00;
        let timestamp_formateado =
            CommitObj::format_timestamp(timestamp, offset_horas, offset_minutos).unwrap();
        let timestamp_formateado_esperado = "Fri Feb 13 20:31:30 2009 -0300";
        assert_eq!(timestamp_formateado, timestamp_formateado_esperado);
    }

    #[test]
    fn test02_formatear_log() {
        let hash_commit = "1a2b3c4d5e6f7g8h9i0j1k2l3m4n5o6p7q8r9s0t".to_string();

        let objeto = CommitObj {
            hash: hash_commit.clone(),
            autor: "nombre_apellido".to_string(),
            mail: "mail".to_string(),
            date: Date {
                tiempo: "1234567890".to_string(),
                offset: "-0300".to_string(),
            },
            mensaje: "Mensaje del commit".to_string(),
            padres: vec![],
            hash_tree: "1a2b3c4d5e6f7g8h9i0j1k2l3m4n5o6p7q8r9s0t".to_string(),
        };

        let contenido_log = objeto.format_log().unwrap();
        let contenido_log_esperado = "commit 1a2b3c4d5e6f7g8h9i0j1k2l3m4n5o6p7q8r9s0t (master)\nAutor: nombre_apellido <mail>\nDate: Fri Feb 13 20:31:30 2009 -0300\n\n     Mensaje del commit\n";
        assert_eq!(contenido_log, contenido_log_esperado);
    }
}

use std::{fs::OpenOptions, io::Write, path, sync::Arc};

use chrono::TimeZone;
use sha1::{Digest, Sha1};

use crate::{
    io::{self, leer_a_string},
    tipos_de_dato::logger::Logger,
    utils::{
        compresion::comprimir_contenido,
        gir_config::{armar_config_con_mail_y_nombre, conseguir_nombre_y_mail_del_config},
        index::limpiar_archivo_index,
    },
};

use super::{merge::Merge, write_tree};

pub struct Commit {
    logger: Arc<Logger>,
    mensaje: String,
}

fn armar_timestamp_commit() -> Result<String, String> {
    let zona_horaria = match chrono::FixedOffset::west_opt(3 * 3600) {
        Some(zona_horaria) => zona_horaria,
        None => return Err("No se pudo obtener la zona horaria".to_string()),
    };
    let now = match zona_horaria.from_local_datetime(&chrono::Local::now().naive_local()) {
        chrono::LocalResult::Single(now) => now,
        _ => return Err("No se pudo obtener la fecha y hora actual".to_string()),
    };
    let timestamp = now.timestamp();
    let offset_horas = -3;
    let offset_minutos = 0;

    let offset_format = format!("{:-03}{:02}", offset_horas, offset_minutos);
    Ok(format!("{} {}", timestamp, offset_format))
}

impl Commit {
    pub fn from(args: &mut Vec<String>, logger: Arc<Logger>) -> Result<Commit, String> {
        if args.is_empty() && Merge::hay_merge_en_curso()? {
            if Merge::hay_archivos_sin_mergear(logger.clone())? {
                return Err("Hay archivos sin mergear".to_string());
            }
            return Commit::from_merge(logger);
        }

        if args.len() != 2 {
            return Err("La cantidad de argumentos es invalida".to_string());
        }
        let mensaje = args
            .pop()
            .ok_or_else(|| "No se especifico un mensaje luego del flag -m".to_string())?;
        let flag = args
            .pop()
            .ok_or_else(|| "No se especifico un flag".to_string())?;
        if flag != "-m" {
            return Err(format!("Flag desconocido {}", flag));
        }
        Ok(Commit { mensaje, logger })
    }

    pub fn from_merge(logger: Arc<Logger>) -> Result<Commit, String> {
        let mensaje = leer_a_string(path::Path::new(".gir/COMMIT_EDITMSG"))?;
        Ok(Commit { mensaje, logger })
    }

    fn hashear_contenido_objeto(&self, contenido: &str) -> String {
        let mut hasher = Sha1::new();
        hasher.update(contenido);
        let hash = hasher.finalize();
        format!("{:x}", hash)
    }

    fn formatear_contenido_commit(
        &self,
        hash_arbol: String,
        hash_padre_commit: String,
    ) -> Result<String, String> {
        let mut contenido_commit = String::new();
        contenido_commit.push_str(&format!("tree {}\n", hash_arbol));
        if !hash_padre_commit.is_empty() {
            contenido_commit.push_str(&format!("parent {}\n", hash_padre_commit));
        }
        if Merge::hay_merge_en_curso()? {
            let padre_mergeado = leer_a_string(path::Path::new(".gir/MERGE_HEAD"))?;
            contenido_commit.push_str(&format!("parent {}\n", padre_mergeado));
        }
        let (nombre, mail) = conseguir_nombre_y_mail_del_config()?;
        let linea_autor = format!("{} <{}>", nombre, mail);
        let timestamp = armar_timestamp_commit()?;
        contenido_commit.push_str(&format!(
            "author {} {}\ncommitter {} {}\n\n{}",
            linea_autor, timestamp, linea_autor, timestamp, self.mensaje
        ));
        Ok(contenido_commit)
    }

    fn crear_contenido_commit(
        &self,
        hash_padre_commit: String,
    ) -> Result<(String, String), String> {
        let hash_arbol = match hash_padre_commit.as_str() {
            "" => write_tree::crear_arbol_commit(None, self.logger.clone())?,
            _ => write_tree::crear_arbol_commit(
                Some(hash_padre_commit.clone()),
                self.logger.clone(),
            )?,
        };
        let contenido_commit =
            self.formatear_contenido_commit(hash_arbol.clone(), hash_padre_commit)?;
        let header = format!("commit {}\0", contenido_commit.len());
        let contenido_total = format!("{}{}", header, contenido_commit);
        Ok((hash_arbol, contenido_total))
    }

    fn escribir_objeto_commit(hash: String, contenido_comprimido: Vec<u8>) -> Result<(), String> {
        let ruta = format!(".gir/objects/{}/{}", &hash[..2], &hash[2..]);
        io::escribir_bytes(ruta, contenido_comprimido)?;
        Ok(())
    }

    pub fn obtener_branch_actual() -> Result<String, String> {
        let ruta_branch = leer_a_string(path::Path::new(".gir/HEAD"))?;
        let branch = ruta_branch
            .split('/')
            .last()
            .ok_or_else(|| "No se pudo obtener el nombre del branch".to_string())?;
        Ok(branch.to_string())
    }

    pub fn obtener_ruta_branch_commit() -> Result<String, String> {
        let branch = Self::obtener_branch_actual()?;
        let ruta = format!(".gir/refs/heads/{}", branch);
        Ok(ruta)
    }

    pub fn obtener_hash_del_padre_del_commit() -> Result<String, String> {
        let ruta = Self::obtener_ruta_branch_commit()?;
        let padre_commit = leer_a_string(path::Path::new(&ruta)).unwrap_or_else(|_| "".to_string());
        Ok(padre_commit)
    }

    fn updatear_ref_head(&self, hash: String) -> Result<(), String> {
        let ruta = Self::obtener_ruta_branch_commit()?;

        let mut f = match OpenOptions::new().write(true).truncate(true).open(ruta) {
            Ok(f) => f,
            Err(_) => return Err("No se pudo abrir el archivo head/ref solicitado".to_string()),
        };
        match f.write_all(hash.as_bytes()) {
            Ok(_) => (),
            Err(_) => {
                return Err("No se pudo escribir en el archivo head/ref solicitado".to_string())
            }
        };
        Ok(())
    }

    fn ejecutar_wrapper(&self, contenido_total: String) -> Result<(), String> {
        let contenido_comprimido = comprimir_contenido(contenido_total.clone())?;
        let hash = self.hashear_contenido_objeto(&contenido_total);
        Self::updatear_ref_head(self, hash.clone())?;
        Self::escribir_objeto_commit(hash.clone(), contenido_comprimido)?;
        self.logger.log(format!(
            "commit {}\n Author: {}\n{} ",
            hash, "", self.mensaje
        ));
        limpiar_archivo_index()?;
        Ok(())
    }

    pub fn ejecutar(&self) -> Result<String, String> {
        armar_config_con_mail_y_nombre()?;
        let hash_padre_commit = Self::obtener_hash_del_padre_del_commit()?;
        let (hash_arbol, contenido_total) = self.crear_contenido_commit(hash_padre_commit)?;
        match self.ejecutar_wrapper(contenido_total) {
            Ok(_) => (),
            Err(_) => {
                let _ = std::fs::remove_file(format!(
                    ".gir/objects/{}/{}",
                    &hash_arbol[..2],
                    &hash_arbol[2..]
                ));
                return Err("No se pudo ejecutar el commit".to_string());
            }
        };
        Merge::limpiar_merge_post_commit()?;
        Ok("Commit creado".to_string())
    }
}

#[cfg(test)]
mod tests {
    use std::{path::PathBuf, sync::Arc};

    use crate::{
        io::{self, escribir_bytes, rm_directorio},
        tipos_de_dato::{
            comandos::{add::Add, hash_object::HashObject, init::Init},
            logger::Logger,
        },
        utils::compresion::{descomprimir_objeto, descomprimir_objeto_gir},
    };

    use super::Commit;

    fn craer_archivo_config_default() {
        let home = std::env::var("HOME").unwrap();
        let config_path = format!("{home}/.girconfig");
        let contenido = "nombre =aaaa\nmail =bbbb\n".to_string();
        escribir_bytes(config_path, contenido).unwrap();
    }

    fn limpiar_archivo_gir() {
        rm_directorio(".gir").unwrap();
        let logger = Arc::new(Logger::new(PathBuf::from("tmp/branch_init")).unwrap());
        let init = Init {
            path: "./.gir".to_string(),
            logger,
        };
        init.ejecutar().unwrap();
        craer_archivo_config_default();
    }

    fn conseguir_hash_padre(branch: String) -> String {
        let hash = std::fs::read_to_string(format!(".gir/refs/heads/{}", branch)).unwrap();
        let contenido = descomprimir_objeto(hash.clone(), String::from(".gir/objects/")).unwrap();
        let lineas_sin_null = contenido.replace('\0', "\n");
        let lineas = lineas_sin_null.split('\n').collect::<Vec<&str>>();
        let linea_supuesto_padre = lineas[2].split(' ').collect::<Vec<&str>>();
        let hash_padre = match linea_supuesto_padre[0] {
            "parent" => linea_supuesto_padre[1],
            _ => "",
        };
        hash_padre.to_string()
    }

    fn conseguir_arbol_commit(branch: String) -> String {
        let hash_hijo = io::leer_a_string(format!(".gir/refs/heads/{}", branch)).unwrap();
        let contenido_hijo =
            descomprimir_objeto(hash_hijo.clone(), String::from(".gir/objects/")).unwrap();
        let lineas_sin_null = contenido_hijo.replace('\0', "\n");
        let lineas = lineas_sin_null.split('\n').collect::<Vec<&str>>();
        let arbol_commit = lineas[1];
        let lineas = arbol_commit.split(' ').collect::<Vec<&str>>();
        let arbol_commit = lineas[1];
        arbol_commit.to_string()
    }

    fn addear_archivos_y_comittear(args: Vec<String>, logger: Arc<Logger>) {
        let mut add = Add::from(args, logger.clone()).unwrap();
        add.ejecutar().unwrap();
        let commit =
            Commit::from(&mut vec!["-m".to_string(), "mensaje".to_string()], logger).unwrap();
        commit.ejecutar().unwrap();
    }

    #[test]
    fn test01_se_actualiza_el_head_ref_correspondiente_con_el_hash_del_commit() {
        limpiar_archivo_gir();
        let logger = Arc::new(Logger::new(PathBuf::from("tmp/commit_test01")).unwrap());
        let mut add = Add::from(vec!["test_file.txt".to_string()], logger.clone()).unwrap();
        add.ejecutar().unwrap();
        let commit =
            Commit::from(&mut vec!["-m".to_string(), "mensaje".to_string()], logger).unwrap();
        commit.ejecutar().unwrap();
        let arbol_last_commit = conseguir_arbol_commit("master".to_string());
        assert_eq!(
            arbol_last_commit,
            "ce0ef9a25817847d31d12df1295248d24d07b309".to_string()
        );
    }

    #[test]
    fn test02_al_hacer_dos_commits_el_primero_es_padre_del_segundo() {
        limpiar_archivo_gir();
        let logger = Arc::new(Logger::new(PathBuf::from("tmp/commit_test02")).unwrap());

        addear_archivos_y_comittear(vec!["test_file.txt".to_string()], logger.clone());

        let hash_padre = io::leer_a_string(".gir/refs/heads/master").unwrap();
        addear_archivos_y_comittear(vec!["test_file2.txt".to_string()], logger.clone());

        let hash_padre_desde_hijo = conseguir_hash_padre("master".to_string());
        assert_eq!(hash_padre_desde_hijo, hash_padre.to_string().to_string());
    }

    #[test]
    fn test03_al_hacer_commit_apunta_al_arbol_correcto() {
        limpiar_archivo_gir();
        let logger = Arc::new(Logger::new(PathBuf::from("tmp/commit_test03")).unwrap());
        addear_archivos_y_comittear(vec!["test_file.txt".to_string()], logger);

        let hash_arbol = conseguir_arbol_commit("master".to_string());
        let contenido_arbol =
            descomprimir_objeto(hash_arbol, String::from(".gir/objects/")).unwrap();

        assert_eq!(
            contenido_arbol,
            "tree 41\0100644 test_file.txt\0678e12dc5c03a7cf6e9f64e688868962ab5d8b65".to_string()
        );
    }

    #[test]
    fn test04_al_hacer_commit_de_un_archivo_y_luego_hacer_otro_commit_de_ese_archivo_modificado_el_hash_tree_es_correcto(
    ) {
        limpiar_archivo_gir();
        let logger = Arc::new(Logger::new(PathBuf::from("tmp/commit_test04")).unwrap());
        addear_archivos_y_comittear(vec!["test_file.txt".to_string()], logger.clone());

        escribir_bytes("test_file.txt", "hola").unwrap();
        addear_archivos_y_comittear(vec!["test_file.txt".to_string()], logger.clone());

        let hash_arbol = conseguir_arbol_commit("master".to_string());
        let contenido_arbol = descomprimir_objeto_gir(hash_arbol).unwrap();
        let hash_correcto =
            HashObject::from(&mut vec!["test_file.txt".to_string()], logger.clone())
                .unwrap()
                .ejecutar()
                .unwrap();

        escribir_bytes("test_file.txt", "test file modified").unwrap();
        assert_eq!(
            contenido_arbol,
            format!("tree 41\0100644 test_file.txt\0{}", hash_correcto)
        );
    }

    #[test]
    fn test05_al_hacer_commit_de_un_directorio_y_luego_hacer_otro_commit_de_ese_directorio_modificado_el_hash_tree_es_correcto(
    ) {
        limpiar_archivo_gir();
        let logger = Arc::new(Logger::new(PathBuf::from("tmp/commit_test05")).unwrap());
        addear_archivos_y_comittear(vec!["test_dir/muchos_objetos".to_string()], logger.clone());

        escribir_bytes("test_dir/muchos_objetos/archivo.txt", "hola").unwrap();
        addear_archivos_y_comittear(vec!["test_dir/muchos_objetos".to_string()], logger.clone());

        let hash_arbol = conseguir_arbol_commit("master".to_string());
        let hash_arbol_git = "c847ae43830604fea16a9830f90e60f0a5f0d993";

        escribir_bytes("test_dir/muchos_objetos/archivo.txt", "mas contenido").unwrap();
        assert_eq!(hash_arbol_git, hash_arbol);
    }
}

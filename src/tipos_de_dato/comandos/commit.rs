use std::{fs::OpenOptions, io::Write, path, rc::Rc};

use sha1::{Digest, Sha1};

use crate::{
    io::{self, leer_a_string},
    tipos_de_dato::{logger::Logger, utilidades_index},
    utilidades_de_compresion::comprimir_contenido,
};

use super::write_tree;

pub struct Commit {
    logger: Rc<Logger>,
    mensaje: String,
}

impl Commit {
    pub fn from(args: &mut Vec<String>, logger: Rc<Logger>) -> Result<Commit, String> {
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

    fn hashear_contenido_objeto(&self, contenido: &str) -> String {
        let mut hasher = Sha1::new();
        hasher.update(contenido);
        let hash = hasher.finalize();
        format!("{:x}", hash)
    }

    fn crear_contenido_commit(
        &self,
        hash_padre_commit: String,
    ) -> Result<(String, String), String> {
        let hash_arbol = match hash_padre_commit.as_str() {
            "" => write_tree::crear_arbol_commit(None)?,
            _ => write_tree::crear_arbol_commit(Some(hash_padre_commit.clone()))?,
        };

        let contenido_commit = format!(
            "tree {}\nparent {}\nauthor {}\ncommitter {}\n\n{}",
            hash_arbol, hash_padre_commit, "", "", self.mensaje
        );
        let header = format!("commit {}\0", contenido_commit.len());
        let contenido_total = format!("{}{}", header, contenido_commit);

        Ok((hash_arbol, contenido_total))
    }

    fn escribir_objeto_commit(hash: String, contenido_comprimido: Vec<u8>) -> Result<(), String> {
        let ruta = format!(".gir/objects/{}/{}", &hash[..2], &hash[2..]);
        io::escribir_bytes(&ruta, contenido_comprimido)?;
        Ok(())
    }

    pub fn obtener_branch_actual() -> Result<String, String> {
        let ruta_branch = leer_a_string(path::Path::new(".gir/HEAD"))?;
        let branch = ruta_branch
            .split("/")
            .last()
            .ok_or_else(|| "No se pudo obtener el nombre del branch".to_string())?;
        Ok(branch.to_string())
    }

    fn obtener_ruta_branch_commit() -> Result<String, String> {
        let branch = Self::obtener_branch_actual()?;
        let ruta = format!(".gir/refs/heads/{}", branch);
        Ok(ruta)
    }

    fn obtener_hash_del_padre_del_commit() -> Result<String, String> {
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
        Self::updatear_ref_head(&self, hash.clone())?;
        Self::escribir_objeto_commit(hash.clone(), contenido_comprimido)?;
        self.logger.log(format!(
            "commit {}\n Author: {}\n{} ",
            hash, "", self.mensaje
        ));
        utilidades_index::limpiar_archivo_index()?;
        Ok(())
    }

    pub fn ejecutar(&self) -> Result<(), String> {
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
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use std::{path::PathBuf, rc::Rc};

    use crate::{
        io::rm_directorio,
        tipos_de_dato::{
            comandos::{add::Add, init::Init},
            logger::Logger,
        },
        utilidades_de_compresion,
    };

    use super::Commit;

    fn limpiar_archivo_gir() {
        rm_directorio(".gir").unwrap();
        let logger = Rc::new(Logger::new(PathBuf::from("tmp/branch_init")).unwrap());
        let init = Init {
            path: "./.gir".to_string(),
            logger,
        };
        init.ejecutar().unwrap();
    }

    fn conseguir_hash_padre(branch: String) -> String {
        let hash = std::fs::read_to_string(format!(".gir/refs/heads/{}", branch)).unwrap();
        let contenido = utilidades_de_compresion::descomprimir_objeto(hash.clone()).unwrap();
        let lineas_sin_null = contenido.replace("\0", "\n");
        let lineas = lineas_sin_null.split("\n").collect::<Vec<&str>>();
        let hash_padre = lineas[2];
        hash_padre.to_string()
    }

    fn conseguir_arbol_commit(branch: String) -> String {
        let hash_hijo = std::fs::read_to_string(format!(".gir/refs/heads/{}", branch)).unwrap();
        let contenido_hijo =
            utilidades_de_compresion::descomprimir_objeto(hash_hijo.clone()).unwrap();
        let lineas_sin_null = contenido_hijo.replace("\0", "\n");
        let lineas = lineas_sin_null.split("\n").collect::<Vec<&str>>();
        let arbol_commit = lineas[1];
        let lineas = arbol_commit.split(" ").collect::<Vec<&str>>();
        let arbol_commit = lineas[1];
        arbol_commit.to_string()
    }

    fn addear_archivos_y_comittear(args: Vec<String>, logger: Rc<Logger>) {
        let mut add = Add::from(args, logger.clone()).unwrap();
        add.ejecutar().unwrap();
        let commit =
            Commit::from(&mut vec!["-m".to_string(), "mensaje".to_string()], logger).unwrap();
        commit.ejecutar().unwrap();
    }

    #[test]
    fn test01_se_actualiza_el_head_ref_correspondiente_con_el_hash_del_commit() {
        limpiar_archivo_gir();
        let logger = Rc::new(Logger::new(PathBuf::from("tmp/commit_test01")).unwrap());
        let mut add = Add::from(vec!["test_file.txt".to_string()], logger.clone()).unwrap();
        add.ejecutar().unwrap();
        let commit =
            Commit::from(&mut vec!["-m".to_string(), "mensaje".to_string()], logger).unwrap();
        commit.ejecutar().unwrap();
        let contenido_head_ref = std::fs::read_to_string(".gir/refs/heads/master").unwrap();
        assert_eq!(
            contenido_head_ref,
            "b7bc8f86f54d688762276aa50bfec4cffafdcc01".to_string()
        );
    }

    #[test]
    fn test02_al_hacer_dos_commits_el_primero_es_padre_del_segundo() {
        limpiar_archivo_gir();
        let logger = Rc::new(Logger::new(PathBuf::from("tmp/commit_test02")).unwrap());

        addear_archivos_y_comittear(vec!["test_file.txt".to_string()], logger.clone());

        let hash_padre = std::fs::read_to_string(".gir/refs/heads/master").unwrap();
        addear_archivos_y_comittear(vec!["test_file.txt".to_string()], logger.clone());

        let hash_padre_desde_hijo = conseguir_hash_padre("master".to_string());

        assert_eq!(
            hash_padre_desde_hijo,
            format!("parent {}", hash_padre).to_string()
        );
        let contenido_head_ref = std::fs::read_to_string(".gir/refs/heads/master").unwrap();
        assert_eq!(
            contenido_head_ref,
            "3180b84bdbe898f3c72643981abdcbe02c63f1e7".to_string()
        );
    }

    #[test]
    fn test03_al_hacer_commit_apunta_al_arbol_correcto() {
        limpiar_archivo_gir();
        let logger = Rc::new(Logger::new(PathBuf::from("tmp/commit_test03")).unwrap());
        addear_archivos_y_comittear(vec!["test_file.txt".to_string()], logger);

        let hash_arbol = conseguir_arbol_commit("master".to_string());
        let contenido_arbol = utilidades_de_compresion::descomprimir_objeto(hash_arbol).unwrap();

        assert_eq!(
            contenido_arbol,
            "tree 41\0100644 test_file.txt\0678e12dc5c03a7cf6e9f64e688868962ab5d8b65".to_string()
        );
    }
}

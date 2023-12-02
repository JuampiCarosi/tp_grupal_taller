use std::{path::PathBuf, sync::Arc};

use crate::{
    tipos_de_dato::logger::Logger,
    utils::{self, io},
};

pub struct CheckIgnore {
    logger: Arc<Logger>,
    paths: Vec<String>,
}

impl CheckIgnore {
    pub fn es_directorio_a_ignorar(
        ubicacion: &PathBuf,
        logger: Arc<Logger>,
    ) -> Result<bool, String> {
        let path = ubicacion
            .to_str()
            .ok_or_else(|| "Path invalido".to_string())?;

        let path_a_verificar = vec![path.to_string()];
        let check_ignore = CheckIgnore::from(path_a_verificar, logger)?;
        let archivos_ignorados = check_ignore.ejecutar()?;
        if !archivos_ignorados.is_empty() {
            return Ok(true);
        }
        Ok(false)
    }

    pub fn from(args: Vec<String>, logger: Arc<Logger>) -> Result<CheckIgnore, String> {
        if args.is_empty() {
            return Err("Ingrese la ruta del archivo buscado como parametro".to_string());
        }
        let paths = args;
        Ok(CheckIgnore { logger, paths })
    }

    pub fn ejecutar(&self) -> Result<String, String> {
        self.logger.log("Buscando archivos ignorados");
        let archivos_ignorados = match io::leer_a_string(".girignore") {
            Ok(archivos_ignorados) => archivos_ignorados,
            Err(_) => return Ok("".to_string()),
        };
        if archivos_ignorados.is_empty() {
            return Ok("".to_string());
        }

        let archivos_ignorados_separados: Vec<PathBuf>;
        archivos_ignorados_separados = archivos_ignorados
            .split("\n")
            .map(|x| PathBuf::from(x))
            .collect();

        let mut archivos_encontrados: Vec<String> = Vec::new();

        for path in &self.paths {
            if utils::path_buf::esta_directorio_habilitado(
                &PathBuf::from(path),
                &archivos_ignorados_separados,
            ) {
                archivos_encontrados.push(path.to_string());
            }
        }

        self.logger.log("Check ignore finalizado");

        Ok(archivos_encontrados.join("\n"))
    }
}

#[cfg(test)]
mod test {
    use std::{path::PathBuf, sync::Arc};

    use crate::{
        tipos_de_dato::{
            comando::Ejecutar,
            comandos::{add::Add, check_ignore::CheckIgnore, init::Init, status::Status},
            logger::Logger,
        },
        utils::{self, io},
    };

    fn settupear_girignore() {
        let archivos_a_ignorar = ".log\n.gitignore\n.vscode\n.girignore\n.log.txt\ntes_dir/";
        io::crear_archivo(".girignore").unwrap();
        io::escribir_bytes(".girignore", archivos_a_ignorar).unwrap();
    }

    fn limpiar_archivo_gir() {
        io::rm_directorio(".gir").unwrap();
        let logger = Arc::new(Logger::new(PathBuf::from("tmp/check_ignore_init")).unwrap());
        let mut init = Init {
            path: "./.gir".to_string(),
            logger,
        };
        init.ejecutar().unwrap();
    }

    #[test]
    fn test01_check_ignore_ignora_un_solo_archivo() {
        settupear_girignore();
        let logger = Arc::new(Logger::new(PathBuf::from("tmp/check_ignore_test01")).unwrap());
        let check_ignore = CheckIgnore::from(vec![".log".to_string()], logger).unwrap();
        let resultado = check_ignore.ejecutar().unwrap();
        assert_eq!(resultado, ".log");
    }

    #[test]
    fn test02_check_ignore_ignora_varios_archivos() {
        let logger = Arc::new(Logger::new(PathBuf::from("tmp/check_ignore_test02")).unwrap());
        let check_ignore = CheckIgnore::from(
            vec![
                ".log".to_string(),
                ".girignore".to_string(),
                "tes_dir/".to_string(),
            ],
            logger,
        )
        .unwrap();
        let resultado = check_ignore.ejecutar().unwrap();
        assert_eq!(resultado, ".log\n.girignore\ntes_dir/");
    }

    #[test]
    fn test03_al_addear_archivos_ignorados_estos_no_se_addean() {
        limpiar_archivo_gir();
        let logger = Arc::new(Logger::new(PathBuf::from("tmp/check_ignore_test03")).unwrap());
        let mut add = Add::from(
            vec![
                ".log".to_string(),
                ".girignore".to_string(),
                "tes_dir/".to_string(),
            ],
            logger.clone(),
        )
        .unwrap();
        add.ejecutar().unwrap();
        let index = utils::index::leer_index(logger.clone()).unwrap();
        assert_eq!(index.len(), 0);
    }

    #[test]
    fn test04_obtener_untrackeados_del_status_ignora_los_archivos_ignorados() {
        let logger = Arc::new(Logger::new(PathBuf::from("tmp/check_ignore_test04")).unwrap());
        let status = Status::from(logger).unwrap();
        let untrackeados = status.obtener_untrackeados().unwrap();
        assert!(!untrackeados.iter().any(|x| x == ".log"));
        assert!(!untrackeados.iter().any(|x| x == ".girignore"));
        assert!(!untrackeados.iter().any(|x| x == "tes_dir"));
    }

    #[test]
    fn test05_ignora_files_dentro_de_directorios_ignorados() {
        let logger = Arc::new(Logger::new(PathBuf::from("tmp/check_ignore_test05")).unwrap());
        let check_ignore =
            CheckIgnore::from(vec!["tes_dir/archivo_dir.txt".to_string()], logger).unwrap();
        let resultado = check_ignore.ejecutar().unwrap();
        assert_eq!(resultado, "tes_dir/archivo_dir.txt");
    }

    #[test]
    fn test06_si_tengo_un_archivo_y_directorio_con_nombres_parecidos_solo_ignora_al_indicado() {
        let logger = Arc::new(Logger::new(PathBuf::from("tmp/check_ignore_test06")).unwrap());
        io::escribir_bytes(".girignore", "test_file/").unwrap();
        let check_ignore = CheckIgnore::from(vec!["test_file.txt".to_string()], logger).unwrap();
        io::crear_directorio("test_file").unwrap();
        let resultado = check_ignore.ejecutar().unwrap();
        io::rm_directorio("test_file").unwrap();
        settupear_girignore();
        assert!(resultado.is_empty());
    }
}

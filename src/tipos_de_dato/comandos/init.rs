use std::{fs, path::Path, sync::Arc};

use crate::{io, tipos_de_dato::logger::Logger};

pub struct Init {
    pub path: String,
    pub logger: Arc<Logger>,
}

impl Init {
    pub fn validar_argumentos(args: Vec<String>) -> Result<(), String> {
        if args.len() > 1 {
            return Err("Argumentos desconocidos\n gir init [<directory>]".to_string());
        }

        Ok(())
    }

    pub fn from(args: Vec<String>, logger: Arc<Logger>) -> Result<Init, String> {
        logger.log(format!("Se intenta crear comando init con args:{:?}", args));

        Self::validar_argumentos(args.clone())?;

        logger.log(format!("Se creo correctamente el comando init:{:?}", args));

        Ok(Init {
            path: Self::obtener_path(args),
            logger,
        })
    }

    pub fn ejecutar(&self) -> Result<String, String> {
        self.logger.log("Se ejecuta init".to_string());

        self.crear_directorio_gir()?;

        let mensaje = format!("Directorio gir creado en {}", self.path);

        self.logger.log(mensaje.clone());

        Ok(mensaje)
    }

    fn obtener_path(args: Vec<String>) -> String {
        if args.is_empty() {
            "./.gir".to_string()
        } else {
            format!("{}{}", args[0], "/.gir")
        }
    }

    fn crear_directorio_gir(&self) -> Result<(), String> {
        if self.verificar_si_ya_esta_creado_directorio_gir() {
            return Err("Ya existe un repositorio en este directorio".to_string());
        };

        if let Err(msj_err) = self.crear_directorios_y_archivos_gir() {
            self.borar_directorios_y_archivos_git();
            return Err(msj_err);
        }

        Ok(())
    }

    fn borar_directorios_y_archivos_git(&self) {
        let _ = fs::remove_dir_all(self.path.clone());
    }

    fn crear_directorios_y_archivos_gir(&self) -> Result<(), String> {
        io::crear_directorio(self.path.clone())?;
        io::crear_directorio(self.path.clone() + "/objects")?;
        io::crear_directorio(self.path.clone() + "/refs/heads")?;
        io::crear_directorio(self.path.clone() + "/refs/tags")?;
        io::crear_directorio(self.path.clone() + "/refs/remotes")?;

        io::crear_archivo(self.path.clone() + "/CONFIG")?;
        io::crear_archivo(self.path.clone() + "/refs/heads/master")?;
        self.crear_archivo_head()
    }

    fn crear_archivo_head(&self) -> Result<(), String> {
        let dir_archivo_head = self.path.clone() + "/HEAD";
        let contenido_inicial_head = "ref: refs/heads/master";

        io::crear_archivo(dir_archivo_head.clone())?;
        io::escribir_bytes(dir_archivo_head, contenido_inicial_head)
    }

    fn verificar_si_ya_esta_creado_directorio_gir(&self) -> bool {
        Path::new(&self.path).exists()
    }
}

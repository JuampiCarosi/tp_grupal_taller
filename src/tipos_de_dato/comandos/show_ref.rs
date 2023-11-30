use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
};

use crate::{
    tipos_de_dato::logger::Logger,
    utils::{io, path_buf},
};

pub struct ShowRef {
    logger: Arc<Logger>,
    show_head: bool,
    show_heads: bool,
    show_remotes: bool,
    show_tags: bool,
}

impl ShowRef {
    pub fn from(args: Vec<String>, logger: Arc<Logger>) -> Result<Self, String> {
        if args.len() == 0 {
            return Ok(ShowRef {
                logger,
                show_head: false,
                show_heads: true,
                show_remotes: true,
                show_tags: true,
            });
        }

        if args.len() == 1 && args[0] == "--head" {
            return Ok(ShowRef {
                logger,
                show_head: true,
                show_heads: true,
                show_remotes: true,
                show_tags: true,
            });
        }

        let mut show_head = false;
        let mut show_heads = false;
        let mut show_tags = false;

        for arg in args {
            match arg.as_str() {
                "--head" => show_head = true,
                "--heads" => show_heads = true,
                "--tags" => show_tags = true,
                _ => return Err(format!("Opcion no conocida '{}'", arg)),
            }
        }

        Ok(ShowRef {
            logger,
            show_head,
            show_heads,
            show_tags,
            show_remotes: false,
        })
    }

    fn hay_que_ver_path(&self, path: &Path) -> Result<bool, String> {
        let nombre = path_buf::obtener_nombre(&path)?;
        let padre = path.ancestors().nth(1).ok_or("Error al obtener el padre")?;

        if &path_buf::obtener_nombre(padre)? != "refs" {
            return Ok(true);
        }

        match nombre.as_str() {
            "heads" => Ok(self.show_heads),
            "remotes" => Ok(self.show_remotes),
            "tags" => Ok(self.show_tags),
            _ => Ok(true),
        }
    }

    fn agregar_head(&self, refs: &mut HashMap<String, String>) -> Result<(), String> {
        let binding = io::leer_a_string(PathBuf::from(".gir/HEAD"))?;
        let head_dir = binding.split(" ").nth(1).ok_or("Error al parsear HEAD")?;
        let contenido = io::leer_a_string(PathBuf::from(format!(".gir/{}", head_dir)))?;
        refs.insert("HEAD".to_string(), contenido);
        Ok(())
    }

    pub fn obtener_referencias(&self, path: PathBuf) -> Result<HashMap<String, String>, String> {
        let mut refs: HashMap<String, String> = HashMap::new();

        let entries = std::fs::read_dir(path)
            .map_err(|e| format!("Error al leer el directorio de refs: {}", e))?;

        for ref_entry in entries {
            let ref_path = ref_entry
                .map_err(|e| format!("Error al leer el directorio de refs: {}", e))?
                .path();

            if !self.hay_que_ver_path(&ref_path)? {
                continue;
            }

            if ref_path.is_dir() {
                let hijos = self.obtener_referencias(ref_path)?;
                refs.extend(hijos);
                continue;
            }

            let contenido_ref = io::leer_a_string(&ref_path)?;

            if contenido_ref.is_empty() {
                return Err(format!("el ref {} esta vacio", ref_path.display()));
            }

            refs.insert(
                ref_path
                    .strip_prefix(".gir/")
                    .unwrap()
                    .display()
                    .to_string(),
                contenido_ref,
            );
        }

        Ok(refs)
    }

    pub fn ejecutar(&self) -> Result<String, String> {
        self.logger.log("Ejecutando comando show-ref");
        let mut refs = self.obtener_referencias(PathBuf::from(".gir/refs/"))?;

        if self.show_head {
            self.agregar_head(&mut refs)?;
        }

        let mut salida = String::new();

        for (ubicacion, contenido) in refs {
            salida.push_str(&format!("{} {}\n", contenido, ubicacion));
        }
        self.logger.log("Comando show-ref ejecutado con exito");
        Ok(salida)
    }
}

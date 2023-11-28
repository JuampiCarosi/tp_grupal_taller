use std::sync::Arc;

use crate::utils::{io, path_buf};

use super::{
    comandos::{commit::Commit},
    logger::Logger,
};

pub struct Tag {
    logger: Arc<Logger>,
    tag_to_create: Option<String>,
}

impl Tag {
    pub fn from(args: Vec<String>, logger: Arc<Logger>) -> Result<Tag, String> {
        if args.is_empty() {
            return Ok(Tag {
                logger,
                tag_to_create: None,
            });
        }

        if args.len() != 1 {
            return Err("Cantidad de argumentos invalida".to_string());
        }

        let tag_to_create = Some(args[0].clone());

        Ok(Tag {
            logger,
            tag_to_create,
        })
    }

    fn obtener_tags(&self) -> Result<Vec<String>, String> {
        let ubicacion = "./.gir/refs/tags";
        let mut tags: Vec<String> = Vec::new();

        let tags_entries = std::fs::read_dir(ubicacion)
            .map_err(|e| format!("Error al leer el directorio de tags: {}", e))?;

        for tag_entry in tags_entries {
            let tag_dir = tag_entry
                .map_err(|e| format!("Error al leer el directorio de tags: {}", e))?
                .path();
            let tag = path_buf::obtener_nombre(&tag_dir)?;

            tags.push(tag);
        }

        Ok(tags)
    }

    fn crear_tag(&self, tag: &str) -> Result<(), String> {
        let tags = self.obtener_tags()?;

        if tags.contains(&tag.to_string()) {
            return Err(format!("El tag {} ya existe", tag));
        }

        let ubicacion = format!(".gir/refs/tags/{}", tag);
        let commit = Commit::obtener_hash_del_padre_del_commit()?;

        io::escribir_bytes(ubicacion, commit)?;

        self.logger.log(&format!("Tag {} creado con exito", tag));

        Ok(())
    }

    pub fn ejecutar(&self) -> Result<String, String> {
        match &self.tag_to_create {
            Some(tag_name) => {
                self.crear_tag(tag_name)?;
                Ok(String::new())
            }
            None => {
                let tags = self.obtener_tags()?;
                Ok(tags.join("\n"))
            }
        }
    }
}

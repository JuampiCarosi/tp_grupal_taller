use std::sync::Arc;

use crate::utils::{self, io};

use super::{comandos::commit::Commit, logger::Logger};

pub struct Tag {
    logger: Arc<Logger>,
    tag_to_create: Option<String>,
}

impl Tag {
    pub fn from(args: Vec<String>, logger: Arc<Logger>) -> Result<Tag, String> {
        if args.len() == 0 {
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
        utils::tags::obtener_tags()
    }

    fn crear_tag(&self, tag: &str) -> Result<(), String> {
        if utils::tags::existe_tag(&tag.to_string()) {
            return Err(format!("El tag {} ya existe", tag));
        }

        let ubicacion = format!(".gir/refs/tags/{}", tag);
        let commit = Commit::obtener_hash_commit_actual()?;

        io::escribir_bytes(ubicacion, &commit)?;

        self.logger.log(&format!("Tag {} creado con exito", tag));

        Ok(())
    }

    pub fn ejecutar(&self) -> Result<String, String> {
        match &self.tag_to_create {
            Some(tag_name) => {
                self.crear_tag(&tag_name)?;
                Ok(String::new())
            }
            None => {
                let tags = self.obtener_tags()?;
                Ok(tags.join("\n"))
            }
        }
    }
}

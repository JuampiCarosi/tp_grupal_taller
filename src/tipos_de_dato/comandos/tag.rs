use std::sync::Arc;

use crate::{
    tipos_de_dato::{comando::Ejecutar, logger::Logger},
    utils::{self, io},
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
        utils::tags::obtener_tags()
    }

    fn crear_tag(&self, tag: &str) -> Result<(), String> {
        if utils::tags::existe_tag(&tag.to_string()) {
            return Err(format!("El tag {} ya existe", tag));
        }

        let ubicacion = format!(".gir/refs/tags/{}", tag);
        let commit = utils::ramas::obtner_commit_head_rama_acutual()?;

        io::escribir_bytes(ubicacion, commit)?;

        self.logger.log(&format!("Tag {} creado con exito", tag));

        Ok(())
    }
}

impl Ejecutar for Tag {
    fn ejecutar(&mut self) -> Result<String, String> {
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

use std::sync::Arc;

use crate::{
    tipos_de_dato::logger::Logger,
    utils::{self, io, ramas},
};

pub struct Tag {
    logger: Arc<Logger>,
    tag_to_create: Option<String>,
}

impl Tag {
    /// Devuelve un Tag con los parametros ingresados por el usuario.
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

    /// Devuelve un vector con los nombres de los tags existentes dentro del repositorio.
    /// Si no hay tags, devuelve un vector vacio.
    fn obtener_tags(&self) -> Result<Vec<String>, String> {
        utils::tags::obtener_tags()
    }

    /// Crea un tag con el nombre ingresado por el usuario.
    /// Si el tag ya existe, devuelve un error.
    fn crear_tag(&self, tag: &str) -> Result<(), String> {
        if utils::tags::existe_tag(&tag.to_string()) {
            return Err(format!("El tag {} ya existe", tag));
        }

        let ubicacion = format!(".gir/refs/tags/{}", tag);
        let commit = ramas::obtener_hash_commit_asociado_rama_actual()?;

        io::escribir_bytes(ubicacion, commit)?;

        self.logger.log(&format!("Tag {} creado con exito", tag));

        Ok(())
    }

    /// Ejecuta el comando tag.
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

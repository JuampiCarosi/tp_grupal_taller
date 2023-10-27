use std::{collections::HashMap, rc::Rc};

use crate::{io::leer_a_string, tipos_de_dato::logger::Logger};

enum ComandoRemote {
    Mostrar,
    Agregar,
    Eliminar,
    CambiarUrl,
    MostrarUrl,
}

pub struct Remote {
    comando: ComandoRemote,
    nombre: Option<String>,
    url: Option<String>,
    logger: Rc<Logger>,
}

const INPUT_ERROR: &str = "gir remote add [<nombre-remote>] [<url-remote>]\ngir remote delete [<nombre-remote>] [<url-remote>]\ngir remote set-url [<nombre-remote>] [<url-remote>]\ngir remote show-url [<nombre-remote>] [<url-remote>]";

impl Remote {
    pub fn from(args: &mut Vec<String>, logger: Rc<Logger>) -> Result<Remote, String> {
        if args.len() > 3 {
            return Err(format!("Demasiados argumentos\n{}", INPUT_ERROR));
        }

        if args.len() == 0 {
            return Ok(Remote {
                comando: ComandoRemote::Mostrar,
                logger,
                nombre: None,
                url: None,
            });
        }

        if args.len() == 2 {
            match args[0].as_str() {
                "show-url" => {
                    return Ok(Remote {
                        comando: ComandoRemote::MostrarUrl,
                        logger,
                        nombre: Some(args[1].clone()),
                        url: None,
                    })
                }
                "delete" => {
                    return Ok(Remote {
                        comando: ComandoRemote::Eliminar,
                        logger,
                        nombre: Some(args[1].clone()),
                        url: None,
                    })
                }
                _ => return Err(INPUT_ERROR.to_string()),
            }
        }

        if args.len() == 3 {
            match args[0].as_str() {
                "add" => {
                    return Ok(Remote {
                        comando: ComandoRemote::Agregar,
                        logger,
                        nombre: Some(args[1].clone()),
                        url: Some(args[2].clone()),
                    })
                }
                "set-url" => {
                    return Ok(Remote {
                        comando: ComandoRemote::CambiarUrl,
                        logger,
                        nombre: Some(args[1].clone()),
                        url: Some(args[2].clone()),
                    })
                }
                _ => return Err(INPUT_ERROR.to_string()),
            }
        };

        Err(INPUT_ERROR.to_string())
    }

    // fn agregar

    pub fn ejecutar(&self) {
        // match &self.comando {
        //     ComandoRemote::Mostrar => self.mostrar(),
        //     ComandoRemote::Agregar => self.agregar(),
        //     ComandoRemote::Eliminar => self.eliminar(),
        //     ComandoRemote::CambiarUrl => self.cambiar_url(),
        //     ComandoRemote::MostrarUrl => self.mostrar_url(),
        // }
    }
}

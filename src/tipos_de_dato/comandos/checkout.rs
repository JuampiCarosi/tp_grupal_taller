use std::rc::Rc;

use crate::{
    io,
    tipos_de_dato::{comandos::branch::Branch, logger::Logger, objetos::tree::Tree, utilidades_index},
    utilidades_path_buf,
};

const PATH_HEAD: &str = "./.gir/HEAD";

pub struct Checkout {
    crear_rama: bool,
    rama_a_cambiar: String,
    logger: Rc<Logger>,
}

impl Checkout {
    fn hay_flags(args: &Vec<String>) -> bool {
        args.len() != 1
    }

    fn verificar_argumentos(args: &Vec<String>) -> Result<(), String> {
        if args.len() > 2 {
            return Err(
                "Argumentos desconocidos.\ngir checkcout [-b] <nombre-rama-cambiar>".to_string(),
            );
        }
        Ok(())
    }

    fn crearse_con_flags(args: Vec<String>, logger: Rc<Logger>) -> Result<Checkout, String> {
        match (args[0].as_str(), args[1].clone()) {
            ("-b", rama) => {
                return Ok(Checkout {
                    crear_rama: true,
                    rama_a_cambiar: rama,
                    logger,
                })
            }
            _ => Err("Argumentos invalidos.\ngir chekcout [-b] <nombre-rama-cambiar>".to_string()),
        }
    }

    pub fn from(args: Vec<String>, logger: Rc<Logger>) -> Result<Checkout, String> {
        Self::verificar_argumentos(&args)?;

        if Self::hay_flags(&args) {
            return Self::crearse_con_flags(args, logger);
        }

        Ok(Checkout {
            crear_rama: false,
            rama_a_cambiar: args[0].to_string(),
            logger,
        })
    }

    fn obtener_ramas(&self) -> Result<Vec<String>, String> {
        let directorio = ".gir/refs/heads";
        let entradas = std::fs::read_dir(directorio)
            .map_err(|e| format!("No se pudo leer el directorio:{}\n {}", directorio, e))?;

        let mut output = Vec::new();

        for entrada in entradas {
            let entrada = entrada
                .map_err(|_| format!("Error al leer entrada el directorio {directorio:#?}"))?;

            let nombre = utilidades_path_buf::obtener_nombre(&entrada.path())?;
            output.push(nombre)
        }

        Ok(output)
    }

    fn verificar_si_la_rama_existe(&self) -> Result<(), String> {
        let ramas = self.obtener_ramas()?;
        for rama in ramas {
            if rama == self.rama_a_cambiar {
                return Ok(());
            }
        }
        Err(format!("Fallo: No existe la rama {}", self.rama_a_cambiar))
    }

    fn conseguir_rama_actual(&self, contenidio_head: String) -> Result<String, String> {
        let partes: Vec<&str> = contenidio_head.split('/').collect();
        let rama_actual = partes
            .last()
            .ok_or_else(|| "Fallo en la lectura de HEAD".to_string())?
            .trim();
        Ok(rama_actual.to_string())
    }
    fn cambiar_ref_en_head(&self) -> Result<(), String> {
        let contenido_head = io::leer_a_string(PATH_HEAD)?;

        let rama_actual = self.conseguir_rama_actual(contenido_head.clone())?;

        let nuevo_head = contenido_head.replace(&rama_actual, &self.rama_a_cambiar);

        io::escribir_bytes(PATH_HEAD, nuevo_head)?;

        Ok(())
    }
    fn cambiar_rama(&self) -> Result<String, String> {
        self.verificar_si_la_rama_existe()?;
        self.cambiar_ref_en_head()?;

        let msg = format!("Se cambio la rama actual a {}", self.rama_a_cambiar);
        self.logger.log(msg.clone());

        Ok(msg)
    }

    fn crear_rama(&self) -> Result<(), String> {
        let msg_branch =  Branch::from(&mut vec![self.rama_a_cambiar.clone()], self.logger.clone())?
            .ejecutar()?;
        print!("{}", msg_branch);
        Ok(())
    }

    fn comprobar_que_no_haya_contenido_index(&self)->Result<(),String>{
        if !utilidades_index::esta_vacio_el_index(){
            Err("Fallo, tiene contendio sin guardar. Por favor, haga commit para no perder los cambios".to_string())
        } else {
            Ok(())
        }
    }
    //si hay contenido en el index no swich
    pub fn ejecutar(&self) -> Result<String, String> {
        self.comprobar_que_no_haya_contenido_index()?;

        if self.crear_rama {
            self.crear_rama()?;
        };

        self.cambiar_rama()
    }

}

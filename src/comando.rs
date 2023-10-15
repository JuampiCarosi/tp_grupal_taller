use crate::comando_base::ComandoBase;


// pub trait EjecutarComando {
    //     fn ejecutar_con(&self, flags: Vec<String>, args: Vec<String>);
// }

pub struct Comando {
    comando_base: ComandoBase, 
    flags: Vec<String>,
    args: Vec<String>,
}

impl Comando {
    pub fn new(input: Vec<String>) -> Result<Comando, String> {
        if Self::argumentos_invalidos(&input) {
            return Err("ERROR: cantidad de argumentos insuficientes".to_string());
        }

        let comando_base = ComandoBase::from(&input[1].clone());

        let (flags, args) = Self::identificar_flags_y_arguementos(input);

        Ok(Comando {
            comando_base,
            flags,
            args,
        })
    }

    pub fn ejecutar(&self) -> Result<(), String> {
        self.comando_base.ejecutar(&self.args, &self.flags)
    }

    fn argumentos_invalidos(input: &Vec<String>) -> bool {
        input.len() < 2 && input[0] != "git"
    }

    fn identificar_flags_y_arguementos(input: Vec<String>) -> (Vec<String>, Vec<String>) {
        let mut flags = Vec::new();
        let mut args = Vec::new();

        for elemento in input.iter().skip(2) {
            if elemento.starts_with('-') {
                flags.push(elemento.clone());
            } else {
                args.push(elemento.clone());
            }
        }

        (flags, args)
    }
}



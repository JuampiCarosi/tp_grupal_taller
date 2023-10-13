pub trait FuncionComando {
    fn ejecutar_con(&self, flags: Vec<String>, args: Vec<String>);
}

pub struct Comando {
    funcion: Box<dyn FuncionComando>,
    flags: Vec<String>,
    args: Vec<String>,
}

impl Comando {
    pub fn new(input: Vec<String>) -> Result<Comando, String> {
        if Self::verificar_argumentos(&input) {
            return Err("ERROR: cantidad de arguemtnos insuficientes".to_string());
        }

        let funcion = Self::seleccionar_funcion(&input[1].clone())?;
        let (flags, args) = Self::identificar_flags_y_arguementos(input);

        Ok(Comando {
            funcion,
            flags,
            args,
        })
    }

    pub fn ejecutar(self) {
        self.funcion.ejecutar_con(self.flags, self.args);
    }

    fn verificar_argumentos(input: &Vec<String>) -> bool {
        input.len() < 2
    }

    fn seleccionar_funcion(funcion: &str) -> Result<Box<dyn FuncionComando>, String> {
        match funcion {
            "i_love_c" => Ok(Box::new(ILOVEC)),
            _ => Err("ERROR:funcion inexsistente".to_string()),
        }
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

pub struct ILOVEC;

impl FuncionComando for ILOVEC {
    fn ejecutar_con(&self, _flags: Vec<String>, _args: Vec<String>) {
        print!("I love c\n");
    }
}

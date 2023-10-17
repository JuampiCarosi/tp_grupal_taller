use std::rc::Rc;

use super::{
    comandos::{init::Init, version::Version},
    logger::Logger,
};

pub enum Comando {
    Init(Init),
    Version(Version),
    Unknown,
}

impl Comando {
    pub fn new(input: Vec<String>, logger: Rc<Logger>) -> Result<Comando, String> {
        let (_, rest) = input.split_first().unwrap();
        let (comando, args) = rest.split_first().unwrap();

        println!("comando: {}", comando);

        let comando = match comando.as_str() {
            "version" => Comando::Version(Version::from(Vec::from(args))?),
            "init" => Comando::Init(Init::from(Vec::from(args), logger)?),
            _ => Comando::Unknown,
        };

        Ok(comando)
    }
}

impl Comando {
    pub fn ejecutar(&self) -> Result<(), String> {
        match self {
            Comando::Init(init) => init.ejecutar(),
            Comando::Version(version) => version.ejecutar(),
            Comando::Unknown => Err("Comando desconocido".to_string()),
        }
    }
}

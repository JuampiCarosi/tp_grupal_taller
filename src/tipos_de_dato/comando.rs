use std::rc::Rc;

use super::{
    git_comandos::{git_init::GitInit, git_version::GitVersion},
    logger::Logger,
};

pub enum Comando {
    GitInit(GitInit),
    GitVersion(GitVersion),
    Unknown,
}

impl Comando {
    pub fn new(input: Vec<String>, logger: Rc<Logger>) -> Result<Comando, String> {
        let (_, rest) = input.split_first().unwrap();
        let (comando, args) = rest.split_first().unwrap();

        println!("comando: {}", comando);

        let comando = match comando.as_str() {
            "version" => Comando::GitVersion(GitVersion::from(Vec::from(args))),
            "init" => Comando::GitInit(GitInit::from(Vec::from(args), logger)?),
            _ => Comando::Unknown,
        };

        Ok(comando)
    }
}

impl Comando {
    pub fn ejecutar(&self) -> Result<(), String> {
        match self {
            Comando::GitInit(git_init) => git_init.ejecutar(),
            Comando::GitVersion(git_version) => git_version.ejecutar(),
            Comando::Unknown => Err("Comando desconocido".to_string()),
        }
    }
}

use std::rc::Rc;

use super::{
    git_comandos::{git_hash_object::GitHashObject, git_init::GitInit, git_version::GitVersion},
    logger::Logger,
};

pub enum Comando {
    GitInit(GitInit),
    GitVersion(GitVersion),
    GitHashObject(GitHashObject),
    Unknown,
}

impl Comando {
    pub fn new(input: Vec<String>, logger: Rc<Logger>) -> Result<Comando, String> {
        let (_, rest) = input.split_first().unwrap();
        let (comando, args) = rest.split_first().unwrap();

        println!("comando: {}", comando);

        let mut vector_args = Vec::from(args);

        let comando = match comando.as_str() {
            "version" => Comando::GitVersion(GitVersion::from(vector_args)?),
            "init" => Comando::GitInit(GitInit::from(vector_args, logger)?),
            "hash-object" => Comando::GitHashObject(GitHashObject::from(&mut vector_args, logger)?),
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
            Comando::GitHashObject(git_hash_object) => git_hash_object.ejecutar(),
            Comando::Unknown => Err("Comando desconocido".to_string()),
        }
    }
}

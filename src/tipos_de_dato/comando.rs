use std::rc::Rc;

use super::{
    comandos::{cat_file::CatFile, hash_object::HashObject, init::Init, version::Version},
    logger::Logger,
};

pub enum Comando {
    Init(Init),
    Version(Version),
    HashObject(HashObject),
    CatFile(CatFile),
    Unknown,
}

impl Comando {
    pub fn new(input: Vec<String>, logger: Rc<Logger>) -> Result<Comando, String> {
        let (_, rest) = input.split_first().unwrap();
        let (comando, args) = rest.split_first().unwrap();

        let mut vector_args = Vec::from(args);

        let comando = match comando.as_str() {
            "version" => Comando::Version(Version::from(vector_args)?),
            "init" => Comando::Init(Init::from(vector_args, logger)?),
            "hash-object" => Comando::HashObject(HashObject::from(&mut vector_args, logger)?),
            "cat-file" => Comando::CatFile(CatFile::from(&mut vector_args, logger)?),
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
            Comando::HashObject(hash_object) => hash_object.ejecutar(),
            Comando::CatFile(cat_file) => cat_file.ejecutar(),
            Comando::Unknown => Err("Comando desconocido".to_string()),
        }
    }
}

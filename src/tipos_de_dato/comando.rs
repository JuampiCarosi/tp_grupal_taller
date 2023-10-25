use std::rc::Rc;

use super::{
    comandos::{
        add::Add, cat_file::CatFile, checkout::Checkout, hash_object::HashObject, init::Init,
        rm::Remove, version::Version,
    },
    logger::Logger,
};

pub enum Comando {
    Init(Init),
    Version(Version),
    HashObject(HashObject),
    CatFile(CatFile),
    Add(Add),
    Remove(Remove),
    Checkout(Checkout),
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
            "add" => Comando::Add(Add::from(vector_args, logger)?),
            "rm" => Comando::Remove(Remove::from(vector_args, logger)?),
            "checkout" => Comando::Checkout(Checkout::from(vector_args, logger)?),
            _ => Comando::Unknown,
        };

        Ok(comando)
    }
}

impl Comando {
    pub fn ejecutar(&mut self) -> Result<String, String> {
        match self {
            Comando::Init(init) => init.ejecutar(),
            Comando::Version(version) => version.ejecutar(),
            Comando::HashObject(hash_object) => hash_object.ejecutar(),
            Comando::CatFile(cat_file) => cat_file.ejecutar(),
            Comando::Add(ref mut add) => add.ejecutar(),
            Comando::Remove(ref mut remove) => remove.ejecutar(),
            Comando::Checkout(ref mut checkout) => checkout.ejecutar(),
            Comando::Unknown => Err("Comando desconocido".to_string()),
        }
    }
}

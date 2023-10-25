use std::rc::Rc;

use super::{
    comandos::{
        add::Add, branch::Branch, cat_file::CatFile, checkout::Checkout, commit::Commit,
        hash_object::HashObject, init::Init, rm::Remove, version::Version, log::Log,
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
    Branch(Branch),
    Commit(Commit),
    Log(Log),
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
            "branch" => Comando::Branch(Branch::from(&mut vector_args, logger)?),
            "checkout" => Comando::Checkout(Checkout::from(vector_args, logger)?),
            "commit" => Comando::Commit(Commit::from(&mut vector_args, logger)?),
            "log" => Comando::Log(Log::from(&mut vector_args, logger)?),
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
            Comando::Branch(ref mut branch) => branch.ejecutar(),
            Comando::Commit(ref mut commit) => commit.ejecutar(),
            Comando::Log(ref mut log) => log.ejecutar(),
            Comando::Unknown => Err("Comando desconocido".to_string()),
        }
    }
}

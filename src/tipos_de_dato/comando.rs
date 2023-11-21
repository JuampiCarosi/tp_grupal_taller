use std::{net::TcpStream, sync::Arc};

use super::{
    comandos::{
        add::Add, branch::Branch, cat_file::CatFile, checkout::Checkout, clone::Clone,
        commit::Commit, fetch::Fetch, hash_object::HashObject, init::Init, log::Log,

        ls_tree::LsTree, merge::Merge, pull::Pull, push::Push, remote::Remote, rm::Remove,
        show_ref::ShowRef, status::Status, version::Version,
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
    Clone(Clone),
    Fetch(Fetch<TcpStream>),
    ShowRef(ShowRef),
    Push(Push),
    Pull(Pull),
    Log(Log),
    Status(Status),
    Remote(Remote),
    Merge(Merge),
    LsTree(LsTree),
    LsFiles(LsFiles),
    Unknown,
}

impl Comando {
    pub fn new(input: Vec<String>, logger: Arc<Logger>) -> Result<Comando, String> {
        let (comando, args) = input.split_first().ok_or("No se ingreso ningun comando")?;

        let mut vector_args = args.to_vec();

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
            "fetch" => Comando::Fetch(Fetch::<TcpStream>::new(vector_args, logger)?),
            "clone" => Comando::Clone(Clone::from(logger)?),
            "push" => Comando::Push(Push::new(logger)?),
            "pull" => Comando::Pull(Pull::from(vector_args, logger)?),
            "log" => Comando::Log(Log::from(&mut vector_args, logger)?),
            "status" => Comando::Status(Status::from(logger)?),
            "remote" => Comando::Remote(Remote::from(&mut vector_args, logger)?),
            "merge" => Comando::Merge(Merge::from(&mut vector_args, logger)?),
            "ls-tree" => Comando::LsTree(LsTree::new(logger, &mut vector_args)?),
            "show-ref" => Comando::ShowRef(ShowRef::from(vector_args, logger)?),
            "ls-files" => Comando::LsFiles(LsFiles::from(logger, &mut vector_args)?),
            _ => Comando::Unknown,
        };

        Ok(comando)
    }

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
            Comando::Fetch(ref mut fetch) => fetch.ejecutar(),
            Comando::Clone(clone) => clone.ejecutar(),
            Comando::Push(push) => push.ejecutar(),
            Comando::Log(ref mut log) => log.ejecutar(),
            Comando::Status(ref mut status) => status.ejecutar(),
            Comando::Remote(ref mut remote) => remote.ejecutar(),
            Comando::Merge(ref mut merge) => merge.ejecutar(),
            Comando::Pull(ref mut pull) => pull.ejecutar(),
            Comando::LsTree(ref mut ls_tree) => ls_tree.ejecutar(),
            Comando::ShowRef(ref mut show_ref) => show_ref.ejecutar(),
            Comando::LsFiles(ref mut ls_files) => ls_files.ejecutar(),
            Comando::Unknown => Err("Comando desconocido".to_string()),
        }
    }
}

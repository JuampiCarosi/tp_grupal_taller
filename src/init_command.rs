use crate::command::Command;
use std::{
    path::Path,
    fs
};

// init puede recibir un directorio como parametro (a implementar).
pub struct InitCommand {}

impl Command for InitCommand {
    fn execute(&self) {
        println!("init cmd");
        if Path::new("./.git").exists() { 
            println!("Ya existe un repositorio en este directorio");
            return;
        }
        fs::create_dir("./.git").unwrap();
        fs::create_dir("./.git/objects").unwrap();
        fs::create_dir_all("./.git/refs/heads").unwrap();
        fs::create_dir_all("./.git/refs/tags").unwrap();
        
    }
}

impl From<Vec<String>>for InitCommand {
    fn from(_: Vec<String>) -> Self {
        InitCommand {}
    }
}



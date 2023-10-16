use super::git_comandos::{
    git_version::GitVersion,
    git_init::GitInit,
    };

pub enum GitComando { 
    Version(GitVersion),
    Init(GitInit),
    Unknown,
}


impl GitComando { 
    
    pub fn ejecutar(&self, args: &Vec<String>, flags: &Vec<String>) -> Result<(), String> {
        match self {
            GitComando::Version(GitVersion) => GitVersion.ejecutar(),
            GitComando::Init(GitInit) => GitInit.ejecutar_con(args, flags)?,
            _ =>  return Err("ERROR: comando no implementado".to_string()),
        }  
        Ok(())
    }
}


impl From<&String> for GitComando {
    fn from(input: &String) -> GitComando {
        match input.as_str() {
            "version" => GitComando::Version(GitVersion),
            "init" => GitComando::Init(GitInit),
            _ => GitComando::Unknown,
        }
    }
}
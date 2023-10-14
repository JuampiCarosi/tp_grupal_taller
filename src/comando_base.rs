use std::{
    path::Path,
    fs, 
};


pub enum ComandoBase { 
    Version,
    InitCommand,
    Unknown,
}


impl ComandoBase { 
    
    pub fn ejecutar(&self, args: &Vec<String>, flags: &[String]) -> Result<(), String> {
        match self {
            ComandoBase::Version => {println!("0.0.1");},
            ComandoBase::InitCommand => {Self::ejecutar_init(args, flags).map_err(|err| format!("{}", err))?;},
            _ =>  {return Err("ERROR: comando no implementado".to_string());},
        }  
        Ok(())
    }
    
    fn ejecutar_init(args: &Vec<String>, _flags: &[String]) -> Result<(), std::io::Error> {
        let path: String = if args.is_empty() {
            "./.git".to_string()
        } else {
            format!("{}{}", args[0], "/.git")
        };
        if Path::new(&path).exists() {
            println!("Ya existe un repositorio en este directorio");
            return Err(std::io::Error::new(std::io::ErrorKind::AlreadyExists, "Ya existe un repositorio en este directorio"));
        }
        fs::create_dir(path.clone())?;
        fs::create_dir(path.clone() + "/objects")?;
        fs::create_dir(path.clone() + "/refs/heads")?;
        fs::create_dir(path.clone() + "/refs/tags")?;

        Ok(())
    }
}


impl From<&String> for ComandoBase {
    fn from(input: &String) -> ComandoBase {
        match input.as_str() {
            "version" => ComandoBase::Version,
            "init" => ComandoBase::InitCommand,
            _ => ComandoBase::Unknown,
        }
    }
}
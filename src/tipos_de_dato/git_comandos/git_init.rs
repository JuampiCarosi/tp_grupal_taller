use std::{
    path::Path,
    fs, 
};
pub struct GitInit;

impl GitInit{

    pub fn ejecutar_con(&self, args: &Vec<String>, _flags: &Vec<String>) ->Result<(), String>{
        let path = self.obtener_path(args);

        self.crear_directorio_git(path).map_err(|err| format!("{}", err))?;

        Ok(())
    }
    
    fn obtener_path(&self, args: &Vec<String>) -> String {
        
        if args.is_empty() {
            "./.git".to_string()
        } else {
            format!("{}{}", args[0], "/.git")
        }
    }
    
    fn crear_directorio_git(&self,path: String)->Result<(), std::io::Error>{
        
        self.verificar_si_ya_esta_creado_directorio_git(&path)?;

        fs::create_dir(path.clone())?;
        fs::create_dir(path.clone() + "/objects")?;
        fs::create_dir(path.clone() + "/refs/heads")?;
        fs::create_dir(path.clone() + "/refs/tags")?;

        Ok(())
    }

    fn verificar_si_ya_esta_creado_directorio_git(&self, path: &String)->Result<(), std::io::Error>{
        if Path::new(&path).exists() {
            println!("Ya existe un repositorio en este directorio");
            return Err(std::io::Error::new(std::io::ErrorKind::AlreadyExists, "Ya existe un repositorio en este directorio"));
        }

        Ok(())
    }

}

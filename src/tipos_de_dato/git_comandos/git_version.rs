pub struct GitVersion;

impl GitVersion{

    pub fn ejecutar(&self){
        Self::imprimir_version();
    }

    fn imprimir_version(){
        println!("git version 0.0.1");
    }
}
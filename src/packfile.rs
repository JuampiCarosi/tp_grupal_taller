use crate::err_comunicacion::ErrorDeComunicacion;
use crate::tipos_de_dato::{
    logger::Logger,
    comandos::cat_file::CatFile,
};
use crate::io;
use std::net::TcpStream;
use std::rc::Rc;
pub struct Packfile {
    objetos: Vec<u8>,
    cant_objetos: u32
}

impl Packfile {
    pub fn new() -> Packfile {
        Packfile {
            objetos: Vec::new(),
            cant_objetos: 0
        }
    }


    // el objeto deberia ser el hash completo (?)
    fn aniadir_objeto(&mut self, objeto: String) -> Result<(), String>{
        let logger = Rc::new(Logger::new()?);
        
        let tamanio_objeto = CatFile::from(&mut vec![objeto.clone(), "-s".to_string()], logger.clone())?.ejecutar()?;
        let tipo_objeto = CatFile::from(&mut vec![objeto.clone(), "-t".to_string()], logger.clone())?.ejecutar()?;
        let tipo: u8 = match tipo_objeto.as_str() {
            "commit" => 0b0010_0000, //1
            "tree" => 0b0100_0000, // 2
            "blob" => 0b0110_0000, // 3
            "tag" => 0b1000_0000, // 4
            _ => {return Err("Tipo de objeto invalido".to_string());}
        };
        
        let tamanio_bytes = (tamanio_objeto.parse::<u8>().unwrap().to_be_bytes()[0] - 1)*7 -4  & 0b0001_1111; // accedo al primer byte de la palabra y me quedo con los ultimos 5 bits

        
        // el n-byte tiene en sus primeros 3 bits el tipo de objeto y en los ultimos 5 el tamanio del objeto encodeado en base 128 (creo que era asi jaja mazzeo me traumaste)
        let n_byte: u8 = tipo | tamanio_bytes;
        self.objetos.push(n_byte);
        self.objetos.push(objeto.len() as u8);
        self.cant_objetos += 1;
        Ok(())
    }


    pub fn obtener_packfile_del_dir(&mut self, dir: String) -> Result<(), ErrorDeComunicacion> {
        // want to iterate over the directory given and add all the objects to the packfile
        let objetos = io::obtener_objetos_del_directorio(dir)?;
        for objeto in objetos {
            self.aniadir_objeto(objeto).unwrap();
        }
        Ok(())
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        // Write the packfile header
        bytes.write_all(b"PACK\0\0\0\2").unwrap();
        bytes.write_all(&self.cant_objetos.to_be_bytes()).unwrap();

        // Write the object data
        bytes.extend_from_slice(&self.objetos);

        // Write the packfile checksum
        let checksum = sha1::Sha1::from(&bytes).digest().bytes();
        bytes.extend_from_slice(&checksum);

        bytes
    }
}
}
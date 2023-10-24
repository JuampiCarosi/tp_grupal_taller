use crate::err_comunicacion::ErrorDeComunicacion;
use crate::tipos_de_dato::{
    logger::Logger,
    comandos::cat_file::conseguir_tamanio,
    comandos::cat_file::conseguir_tipo_objeto,
    comandos::cat_file::conseguir_contenido,

};
use crate::io;
use std::path::PathBuf;
use std::rc::Rc;
use sha1::{Digest, Sha1};
pub struct Packfile {
    objetos: Vec<u8>,
    indice: Vec<u8>,
    cant_objetos: u32
}

impl Packfile {
    pub fn new() -> Packfile {
        Packfile {
            objetos: Vec::new(),
            indice: Vec::new(),
            cant_objetos: 0
        }
    }

    fn aniadir_objeto(&mut self, objeto: String) -> Result<(), String>{
        // let logger = Rc::new(Logger::new(PathBuf::from("log.txt"))?);
        
        // optimizar el hecho de que pido descomprimir 2 veces un archivo
        let tamanio_objeto = conseguir_tamanio(objeto.clone())?.parse::<u64>().unwrap();
        let tipo_objeto = conseguir_tipo_objeto(objeto.clone())?;
        // codifica el tamanio del archivo descomprimido y su tipo en un tipo variable de longitud
        let nbyte = match tipo_objeto.as_str() {
            "commit" => codificar_longitud(tamanio_objeto, 1), //1
            "tree" => codificar_longitud(tamanio_objeto, 2), // 2
            "blob" => codificar_longitud(tamanio_objeto, 3), // 3
            "tag" => codificar_longitud(tamanio_objeto, 4), // 4
            _ => {return Err("Tipo de objeto invalido".to_string());} 
        };

        self.objetos.extend(nbyte); 
        let ruta_objeto = format!("./.git/objects/{}/{}", &objeto[..2], &objeto[2..]);
        let objeto_comprimido = io::leer_bytes(&ruta_objeto).unwrap();
        self.objetos.extend(objeto_comprimido);

        self.cant_objetos += 1;
        Ok(())
    }

    // funcion que recorrer el directorio y aniade los objetos al packfile junto a su indice correspondiente
    fn obtener_objetos_del_dir(&mut self, dir: String) -> Result<(), ErrorDeComunicacion> {
        // let objetos = io::obtener_objetos_del_directorio(dir)?;
        let objetos = vec!["1f96920c3930d2086ebcc27c385d11c6284f8d4b".to_string(), "f328f3d7fdb8b7592a879b504bd637d50cbfce3c".to_string()];
        for objeto in objetos {
            let inicio= self.objetos.len() as u32; // obtengo el len previo a aniadir el objeto
            self.aniadir_objeto(objeto.clone()).unwrap();

            let offset = self.objetos.len() as u32 - inicio; 
            self.indice.extend(&offset.to_be_bytes()); 
            self.indice.extend(objeto.as_bytes()); 
        }
        Ok(())
    }

    pub fn obtener_indice(&mut self) -> Vec<u8> {
        self.indice.clone()
    }
    pub fn obtener_pack(&mut self, dir: String) -> Vec<u8> {
        println!("Despachando packfile");
        self.obtener_objetos_del_dir(dir).unwrap(); 
        let mut packfile = Vec::new();

        // agrego el indice primero
        // packfile.extend(&self.indice);
        // agrego pkt flush para separar el indice del pack
        // packfile.extend("0000".as_bytes());

        // posteriormente el pack
        packfile.extend("PACK".as_bytes());
        packfile.extend(&[0, 0, 0, 2]);
        packfile.extend(&self.cant_objetos.to_be_bytes());
        packfile.extend(&self.objetos);

        // computa el hash SHA-1 del packfile
        // let mut hasher = Sha1::new();
        // hasher.update(&packfile);
        // let hash = hasher.finalize();

        // // aniade el hash al final del packfile
        // packfile.extend(&hash);

        packfile
    }
    
    fn verificar_checksum(packfile: &[u8]) -> bool {
        // Get the expected hash from the end of the packfile
        let expected_hash = &packfile[packfile.len() - 20..];
    
        // Compute the SHA-1 hash of the packfile data
        let mut hasher = Sha1::new();
        hasher.update(&packfile[..packfile.len() - 20]);
        let actual_hash = hasher.finalize();
    
        // Compare the expected hash to the actual hash
        expected_hash == actual_hash.as_slice()
    }
}

pub fn codificar_longitud(tamanio: u64, bits_adicionales: u8) -> Vec<u8> {
    let mut resultado = Vec::new();
    let mut value = tamanio

;

    // Agregar los bits adicionales
    let first_byte = ((bits_adicionales & 0x07) << 4) as u8;
    resultado.push(first_byte | 0x08); // Establecer el bit más significativo a 1

    loop {
        let mut byte = (value & 0x7F) as u8;
        value >>= 7;

        if value > 0 {
            byte |= 0x80;
        }

        resultado.push(byte);

        if value == 0 {
            break;
        }
    }

    resultado
}

pub fn decodificar_longitud(bytes: &[u8]) -> Option<(u64, u8)> {
    let mut tamanio = 0u64;
    let mut shift = 0;
    let mut bits_adicionales = 0u8;

    for (i, &byte) in bytes.iter().enumerate() {
        if i == 0 {
            // Decodificar los bits adicionales del primer byte
            bits_adicionales = (byte & 0xF0) >> 4;
        } else {
            tamanio |= ((byte & 0x7F) as u64) << shift;
            shift += 7;

            if (byte & 0x80) == 0 {
                return Some((tamanio, bits_adicionales));
            }
        }
    }

    None
}

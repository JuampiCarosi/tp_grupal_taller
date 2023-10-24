use crate::err_comunicacion::ErrorDeComunicacion;
use crate::tipos_de_dato::{
    comandos::cat_file::conseguir_tamanio,
    comandos::cat_file::conseguir_tipo_objeto,

};
use crate::io;
use std::net::TcpStream;
use sha1::{Digest, Sha1};
use std::io::Read;
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
        let tamanio_objeto = conseguir_tamanio(objeto.clone())?.parse::<u32>().unwrap();
        let tipo_objeto = conseguir_tipo_objeto(objeto.clone())?;
        // codifica el tamanio del archivo descomprimido y su tipo en un tipo variable de longitud
        let nbyte = match tipo_objeto.as_str() {
            "commit" => codificar_bytes(1, tamanio_objeto), //1
            "tree" => codificar_bytes(2, tamanio_objeto), // 2
            "blob" => codificar_bytes(3,tamanio_objeto), // 3
            "tag" => codificar_bytes(4, tamanio_objeto), // 4
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
        // commit y blob
        let objetos = vec!["8ab3bc50ab8155b55c54a2c4d75afdc910203483".to_string(), "0e0082b1300909b92177ba464ee56bd9e8abc4d3".to_string()];
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

// pub fn codificar_longitud(tamanio: u64, bits_adicionales: u8) -> Vec<u8> {
//     let mut resultado = Vec::new();
//     let mut value = tamanio

// ;

//     // Agregar los bits adicionales
//     let first_byte = ((bits_adicionales & 0x07) << 4) as u8;
//     resultado.push(first_byte | 0x08); // Establecer el bit más significativo a 1

//     loop {
//         let mut byte = (value & 0x7F) as u8;
//         value >>= 7;

//         if value > 0 {
//             byte |= 0x80;
//         }

//         resultado.push(byte);

//         if value == 0 {
//             break;
//         }
//     }

//     resultado
// }

// pub fn decodificar_longitud(bytes: &[u8]) -> Option<(u64, u8)> {
//     let mut tamanio = 0u64;
//     let mut shift = 0;
//     let mut bits_adicionales = 0u8;

//     for (i, &byte) in bytes.iter().enumerate() {
//         if i == 0 {
//             // Decodificar los bits adicionales del primer byte
//             bits_adicionales = (byte & 0xF0) >> 4;
//         } else {
//             tamanio |= ((byte & 0x7F) as u64) << shift;
//             shift += 7;

//             if (byte & 0x80) == 0 {
//                 return Some((tamanio, bits_adicionales));
//             }
//         }
//     }

//     None
// }

// 
pub fn codificar_bytes(tipo: u8, numero: u32) -> Vec<u8> {
    let mut resultado = Vec::new();
    let mut valor = numero;
    println!("codificando al tipo: {:?} y numero: {:?}", tipo, numero);

    // si lo el tamanio del numero es mayor a 4 bits, entonces tengo que poner el bit mas significativo en 1
    let primer_byte: u8 = if valor >> 4 != 0 {
        ((tipo & 0x07) << 4) as u8 | 0x80 | (numero & 0x0F) as u8
    } else {
        ((tipo & 0x07) << 4) as u8 | (numero & 0x0F) as u8

    };

    resultado.push(primer_byte); // Establecer el bit más significativo a 1 y agregar los 4 bits finales 
    valor >>= 4;    
    loop {
        if valor == 0 {
            break;
        }
        let mut byte = (valor & 0x7F) as u8;
        valor >>= 7;

        if valor > 0 {
            byte |= 0x80;
        }
        resultado.push(byte);
    }

    resultado
}

// por que 32 y no 64? porque en la docu dice que no tenemos objetos de mas de 4g (2^32)
pub fn decodificar_bytes(flujo: &mut TcpStream) -> (u8, u32) {
    
    let mut byte = [0; 1];
    let mut numero_decodificado: u32 = 0;
    let mut corrimiento: u32 = 0;
    let mut continua = false;
    
    flujo.read_exact(&mut byte).unwrap();
    // decodifico el primer byte que es distinto
    let tipo = byte[0] >> 4 & 0x07; // deduzco el tipo 
    numero_decodificado = (byte[0] & 0x0f) as u32; // obtengo los primeros 4 bits
    if byte[0] & 0x80 != 0 {
        continua = true;
    }
    corrimiento += 4;
    
    loop {
        if !continua {
            break;
        }
        flujo.read_exact(&mut byte).unwrap();
        if byte[0] & 0x80 == 0 {
            continua = false;
            
        }
        numero_decodificado = numero_decodificado << corrimiento;
        corrimiento += 7;
        numero_decodificado |= (byte[0] & 0x7f) as u32;
    }
    (tipo, numero_decodificado)
}
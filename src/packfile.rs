use std::rc::Rc;

use crate::err_comunicacion::ErrorDeComunicacion;
use crate::tipos_de_dato::logger::Logger;
use crate::{io, utilidades_de_compresion, tipos_de_dato};
use crate::tipos_de_dato::comandos::cat_file;

use sha1::{Digest, Sha1};

pub struct Packfile {
    objetos: Vec<u8>,
    indice: Vec<u8>,
    cant_objetos: u32,
}

impl Packfile {
    pub fn new() -> Packfile {
        Packfile {
            objetos: Vec::new(),
            indice: Vec::new(),
            cant_objetos: 0,
        }
    }

    fn aniadir_objeto(&mut self, objeto: String, dir: &str) -> Result<(), String> {
        // let logger = Rc::new(Logger::new(PathBuf::from("log.txt"))?);
        println!("Aniadiendo objeto: {}", objeto);
        // DESHARCODEAR EL ./.GIR
        let ruta_objeto = format!("{}/{}/{}", dir, &objeto[..2], &objeto[2..]);
        println!("ruta objeto: {}", ruta_objeto);
        let objeto_comprimido = io::leer_bytes(&ruta_objeto).unwrap();
        let tamanio_objeto = utilidades_de_compresion::descomprimir_contenido_u8(&objeto_comprimido)?.len() as u32;
        let tipo_objeto = cat_file::obtener_tipo_objeto_de(&objeto, "./.gir/objects")?;

        // codifica el tamanio del archivo descomprimido y su tipo en un tipo variable de longitud
        let nbyte = match tipo_objeto.as_str() {
            "commit" => codificar_bytes(1, tamanio_objeto), //1
            "tree" => codificar_bytes(2, tamanio_objeto),   // 2
            "blob" => codificar_bytes(3, tamanio_objeto),   // 3
            "tag" => codificar_bytes(4, tamanio_objeto),    // 4
            "ofs_delta" => codificar_bytes(6, tamanio_objeto), // 6
            "ref_delta" => codificar_bytes(7, tamanio_objeto), // 7
            _ => {
                return Err("Tipo de objeto invalido".to_string());
            }
        };

        self.objetos.extend(nbyte);
        self.objetos.extend(objeto_comprimido);

        self.cant_objetos += 1;
        Ok(())
    }

    
    // fijarse en commit que algo se manda incompleto, creo 
    // funcion que recorrer el directorio y aniade los objetos al packfile junto a su indice correspondiente
    fn obtener_objetos_del_dir(&mut self, dir: &str) -> Result<(), ErrorDeComunicacion> {
        // esto porque es un clone, deberia pasarle los objetos que quiero
        let objetos = io::obtener_objetos_del_directorio(dir.to_string() + "objects/")?;
        // --- 
        
        for objeto in objetos {
            let inicio = self.objetos.len() as u32; // obtengo el len previo a aniadir el objeto
            self.aniadir_objeto(objeto.clone(), dir).unwrap();

            let offset = self.objetos.len() as u32 - inicio;
            self.indice.extend(&offset.to_be_bytes());
            self.indice.extend(objeto.as_bytes());
        }
        Ok(())
    }

    pub fn obtener_indice(&mut self) -> Vec<u8> {
        self.indice.clone()
    }
    pub fn obtener_pack_entero(&mut self, dir: &str) -> Vec<u8> {
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


    pub fn obtener_pack_con_archivos(&mut self, objetos: Vec<String>, dir: &str) -> Vec<u8> {
        for objeto in objetos {
            self.aniadir_objeto(objeto, dir).unwrap();
        }
        let mut packfile: Vec<u8> = Vec::new();
        packfile.extend("PACK".as_bytes());
        packfile.extend(&[0, 0, 0, 2]);
        packfile.extend(&self.cant_objetos.to_be_bytes());
        packfile.extend(&self.objetos);
        packfile
    }
}

pub fn codificar_bytes(tipo: u8, numero: u32) -> Vec<u8> {
    let mut resultado = Vec::new();
    let mut valor = numero;
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
// devuelve tipo, tamanio del objeto descomprimido y bytes leidos
pub fn decodificar_bytes(bytes: &mut Vec<u8>) -> (u8, u32, u32) {
    // let mut byte = [0; 1];
    let mut numero_decodificado: u32;
    let mut corrimiento: u32 = 0;
    let mut continua = false;
    let mut bytes_leidos = 0;

    // decodifico el primer byte que es distinto
    let tipo = bytes[0] >> 4 & 0x07; // deduzco el tipo
    numero_decodificado = (bytes[0] & 0x0f) as u32; // obtengo los primeros 4 bits

    if bytes[0] & 0x80 != 0 {
        continua = true;
    }
    bytes.remove(0);
    corrimiento += 4;
    bytes_leidos += 1;
    loop {
        if !continua {
            break;
        }
        // flujo.read_exact(&mut byte).unwrap();
        if bytes[0] & 0x80 == 0 {
            continua = false;
        }
        numero_decodificado |= ((bytes[0] & 0x7f) as u32) << corrimiento;
        corrimiento += 7;
        bytes_leidos += 1;
        bytes.remove(0);
    }
    (tipo, numero_decodificado, bytes_leidos)
}

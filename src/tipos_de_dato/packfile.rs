use crate::err_comunicacion::ErrorDeComunicacion;
use crate::tipos_de_dato::comandos::cat_file;
use crate::tipos_de_dato::logger::Logger;
use crate::utils::compresion;
use crate::utils::{self, io};
use flate2::{Decompress, FlushDecompress};
use sha1::{Digest, Sha1};
use std::convert::TryInto;
use std::path::PathBuf;
use std::str;
use std::sync::Arc;
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
        // println!("Aniadiendo objeto: {}", objeto);
        // println!("la dir que llega: {}", dir);
        // DESHARCODEAR EL ./.GIR
        let ruta_objeto = format!("{}{}/{}", dir, &objeto[..2], &objeto[2..]);
        // println!("ruta objeto: {}", ruta_objeto);
        let _objeto_comprimido = io::leer_bytes(ruta_objeto).unwrap();
        let log = Arc::new(Logger::new(PathBuf::from("log.txt")).unwrap());
        // en este catfile hay cosas hardcodeadas que hay que cambiar :{
        let tamanio_objeto_str = match {
            cat_file::CatFile::from(&mut vec!["-s".to_string(), objeto.clone()], log.clone())
                .unwrap()
                .ejecutar_de(dir)
        } {
            Ok(tamanio) => tamanio,
            Err(_) => {
                return Err(format!(
                    "No se pudo obtener el tamanio del objeto {objeto:#?}"
                ));
            }
        };

        let tamanio_objeto = tamanio_objeto_str.trim().parse::<u32>().unwrap_or(0);

        let tipo_objeto = cat_file::obtener_tipo_objeto_de(&objeto, dir)?;
        // codifica el tamanio del archivo descomprimido y su tipo en un tipo variable de longitud
        let nbyte = match tipo_objeto.as_str() {
            "commit" => codificar_bytes(1, tamanio_objeto),    //1
            "tree" => codificar_bytes(2, tamanio_objeto),      // 2
            "blob" => codificar_bytes(3, tamanio_objeto),      // 3
            "tag" => codificar_bytes(4, tamanio_objeto),       // 4
            "ofs_delta" => codificar_bytes(6, tamanio_objeto), // 6
            "ref_delta" => codificar_bytes(7, tamanio_objeto), // 7
            _ => {
                return Err("Tipo de objeto invalido".to_string());
            }
        };
        let obj =
            utils::compresion::obtener_contenido_comprimido_sin_header_de(objeto.clone(), dir)?;
        // println!("objeto comprimido: {:?}", String::from_utf8(obj));
        self.objetos.extend(nbyte);
        self.objetos.extend(obj);

        self.cant_objetos += 1;
        Ok(())
    }

    // fijarse en commit que algo se manda incompleto, creo
    // funcion que recorrer el directorio y aniade los objetos al packfile junto a su indice correspondiente
    fn obtener_objetos_del_dir(&mut self, dir: &str) -> Result<(), ErrorDeComunicacion> {
        // esto porque es un clone, deberia pasarle los objetos que quiero
        let objetos = io::obtener_objetos_del_directorio(dir.to_string()).unwrap();
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
        let mut hasher = Sha1::new();
        hasher.update(&packfile);
        let hash = hasher.finalize();

        // // aniade el hash al final del packfile
        packfile.extend(&hash);

        packfile
    }

    pub fn verificar_checksum(packfile: &[u8]) -> bool {
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
        // packfile.extend(&[0, 0, 0, 2]);
        packfile.extend(2u32.to_be_bytes());
        packfile.extend(&self.cant_objetos.to_be_bytes());
        packfile.extend(&self.objetos);
        let mut hasher = Sha1::new();
        hasher.update(&packfile);
        let hash = hasher.finalize();

        // // aniade el hash al final del packfile
        packfile.extend(&hash);
        packfile
    }

    pub fn obtener_paquete_y_escribir(
        &mut self,
        bytes: &mut Vec<u8>,
        ubicacion: String,
    ) -> Result<(), ErrorDeComunicacion> {
        // a partir de aca obtengo el paquete
        // println!("cant bytes: {:?}", bytes.len());
        // println!("obteniendo firma");
        let checksum = Self::verificar_checksum(bytes);
        match checksum {
            true => println!("Checksum correcto"),
            false => println!("Checksum incorrecto"),
        }
        let _firma = &bytes[0..4];
        // assert_eq!("PACK", str::from_utf8(&firma).unwrap());
        bytes.drain(0..4);
        let _version = &bytes[0..4];
        // assert_eq!("0002", str::from_utf8(&version)?);

        bytes.drain(0..4);
        // println!("obteniendo largo");
        let largo = &bytes[0..4];
        let largo_packfile: [u8; 4] = largo.try_into().unwrap();
        let largo = u32::from_be_bytes(largo_packfile);
        bytes.drain(0..4);
        let mut contador: u32 = 0;
        while contador < largo {
            // println!("cant bytes: {:?}", bytes.len());
            let (tipo, tamanio, _bytes_leidos) = decodificar_bytes(bytes);

            if tipo == 7 {
                let _hash_obj = &bytes[0..20];
                bytes.drain(0..20);
                let mut objeto_descomprimido = vec![0; tamanio as usize];

                let mut descompresor = Decompress::new(true);

                descompresor
                    .decompress(bytes, &mut objeto_descomprimido, FlushDecompress::None)
                    .unwrap();

                let total_in = descompresor.total_in();

                bytes.drain(0..total_in as usize);
                contador += 1;
                continue;
            }
            println!("tipo: {:?}, tamanio: {}", tipo, tamanio);
            // println!("cant bytes post decodificacion: {:?}", bytes.len());
            // println!("tipo: {:?}", tipo);
            // println!("tamanio: {:?}", tamanio);
            // // -- leo el contenido comprimido --
            let mut objeto_descomprimido = vec![0; tamanio as usize];

            let mut descompresor = Decompress::new(true);

            descompresor
                .decompress(bytes, &mut objeto_descomprimido, FlushDecompress::None)
                .unwrap();

            // calculo el hash
            let objeto = Self::obtener_y_escribir_objeto(tipo, tamanio, &mut objeto_descomprimido);
            let mut hasher = Sha1::new();
            hasher.update(objeto.clone());
            let _hash = hasher.finalize();
            let hash = format!("{:x}", _hash);

            let ruta = format!("{}{}/{}", &ubicacion, &hash[..2], &hash[2..]);

            let _total_out = descompresor.total_out(); // esto es lo que debe matchear el tamanio que se pasa en el header
            let total_in = descompresor.total_in(); // esto es para calcular el offset

            bytes.drain(0..total_in as usize);
            io::escribir_bytes(ruta, compresion::comprimir_contenido_u8(&objeto).unwrap()).unwrap();

            contador += 1;
        }
        bytes.drain(0..20); // el checksum
        Ok(())
    }

    fn obtener_y_escribir_objeto(
        tipo: u8,
        tamanio: u32,
        contenido_descomprimido: &mut Vec<u8>,
    ) -> Vec<u8> {
        let mut header: Vec<u8> = Vec::new();
        match tipo {
            1 => {
                header = format!("{} {}\0", "commit", tamanio).as_bytes().to_vec();
            }
            2 => {
                header = format!("{} {}\0", "tree", tamanio).as_bytes().to_vec();
            }
            3 => {
                header = format!("{} {}\0", "blob", tamanio).as_bytes().to_vec();
            }
            _ => {
                eprintln!("Tipo de objeto invalido");
            }
        }
        header.append(contenido_descomprimido);
        header
    }
}

pub fn codificar_bytes(tipo: u8, numero: u32) -> Vec<u8> {
    let mut resultado = Vec::new();
    let mut valor = numero;
    // si lo el tamanio del numero es mayor a 4 bits, entonces tengo que poner el bit mas significativo en 1
    let primer_byte: u8 = if valor >> 4 != 0 {
        ((tipo & 0x07) << 4) | 0x80 | (numero & 0x0F) as u8
    } else {
        ((tipo & 0x07) << 4) | (numero & 0x0F) as u8
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

// tree ef55aae678e3a636dc72d68f3b10f60b2ad2c306
// author JuaniFIUBA <jperezd@fi.uba.ar> 1698954872 -0300
// committer JuaniFIUBA <jperezd@fi.uba.ar> 1698954872 -0300

// archivezco

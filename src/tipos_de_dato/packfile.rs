use crate::tipos_de_dato::comandos::cat_file;
use crate::tipos_de_dato::logger::Logger;
use crate::tipos_de_dato::objetos::tree::Tree;
use crate::utils::compresion;
use crate::utils::{self, io};
use flate2::{Decompress, FlushDecompress};
use sha1::{Digest, Sha1};
use std::convert::TryInto;
use std::path::PathBuf;
use std::str;
use std::sync::Arc;

const COMMIT: u8 = 1;
const TREE: u8 = 2;
const BLOB: u8 = 3;
const TAG: u8 = 4;
const OFS_DELTA: u8 = 6;
const REF_DELTA: u8 = 7;

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
        let ruta_objeto = format!("{}{}/{}", dir, &objeto[..2], &objeto[2..]);
        let _objeto_comprimido = io::leer_bytes(ruta_objeto)?;
        let log = Arc::new(Logger::new(PathBuf::from("log.txt"))?);
        // en este catfile hay cosas hardcodeadas que hay que cambiar :{
        let tamanio_objeto_str = match {
            cat_file::CatFile::from(&mut vec!["-s".to_string(), objeto.clone()], log.clone())?
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
            "commit" => codificar_bytes(COMMIT, tamanio_objeto),    //1
            "tree" => codificar_bytes(TREE, tamanio_objeto),      // 2
            "blob" => codificar_bytes(BLOB, tamanio_objeto),      // 3
            "tag" => codificar_bytes(TAG, tamanio_objeto),       // 4
            "ofs_delta" => codificar_bytes(OFS_DELTA, tamanio_objeto), // 6
            "ref_delta" => codificar_bytes(REF_DELTA, tamanio_objeto), // 7
            _ => {
                return Err("Tipo de objeto invalido".to_string());
            }
        };
        let obj =
            utils::compresion::obtener_contenido_comprimido_sin_header_de(objeto.clone(), dir)?;
        self.objetos.extend(nbyte);
        self.objetos.extend(obj);

        self.cant_objetos += 1;
        Ok(())
    }

    // fijarse en commit que algo se manda incompleto, creo
    // funcion que recorrer el directorio y aniade los objetos al packfile junto a su indice correspondiente
    fn obtener_objetos_del_dir(&mut self, dir: &str) -> Result<(), String> {
        // esto porque es un clone, deberia pasarle los objetos que quiero
        let objetos = io::obtener_objetos_del_directorio(dir.to_string())?;
        // ---
        for objeto in objetos {
            let inicio = self.objetos.len() as u32; // obtengo el len previo a aniadir el objeto
            self.aniadir_objeto(objeto.clone(), dir)?;

            let offset = self.objetos.len() as u32 - inicio;
            self.indice.extend(&offset.to_be_bytes());
            self.indice.extend(objeto.as_bytes());
        }
        Ok(())
    }

    pub fn obtener_indice(&mut self) -> Vec<u8> {
        self.indice.clone()
    }
    pub fn obtener_pack_entero(&mut self, dir: &str) -> Result<Vec<u8>, String> {
        println!("Despachando packfile");
        self.obtener_objetos_del_dir(dir)?;
        let mut packfile = Vec::new();

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

        Ok(packfile)
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

    pub fn obtener_pack_con_archivos(&mut self, objetos: Vec<String>, dir: &str) -> Result<Vec<u8>, String> {
        for objeto in objetos {
            self.aniadir_objeto(objeto, dir)?;
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
        Ok(packfile)
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

// -------------------------------------------------------------------------------------------------------------------------

fn decodificar_bytes_sin_borrado(bytes: &Vec<u8>, offset: &mut usize) -> (u8, u32) {
    let mut numero_decodificado: u32;
    let mut corrimiento: u32 = 0;
    let mut continua = false;

    // decodifico el primer byte que es distinto
    let tipo = &bytes[*offset] >> 4 & 0x07; // deduzco el tipo
    numero_decodificado = (bytes[*offset] & 0x0f) as u32; // obtengo los primeros 4 bits

    if bytes[*offset] & 0x80 != 0 {
        continua = true;
    }
    *offset += 1;
    // bytes.remove(0);
    corrimiento += 4;
    // bytes_leidos += 1;
    loop {
        if !continua {
            break;
        }
        // flujo.read_exact(&mut byte).unwrap();
        if bytes[*offset] & 0x80 == 0 {
            continua = false;
        }
        numero_decodificado |= ((&bytes[*offset] & 0x7f) as u32) << corrimiento;
        corrimiento += 7;
        *offset += 1;
        // bytes_leidos += 1;
        // bytes.remove(0);
    }
    (tipo, numero_decodificado)
}

fn leer_varint(bytes: &mut Vec<u8>) -> u32 {
    let mut val: u32 = 0;
    loop {
        let byt: u32 = bytes[0] as u32;
        val = (val << 7) | (byt & 0x7f);
        if byt & 0x80 == 0 {
            return val;
        }
        bytes.remove(0);
    }
}

fn leer_varint_sin_consumir_bytes(bytes: &Vec<u8>, offset: &mut usize) -> u32 {
    let mut val: u32 = 0;
    loop {
        let byt: u32 = bytes[*offset] as u32;
        val = (val << 7) | (byt & 0x7f);
        if byt & 0x80 == 0 {
            return val;
        }
        *offset += 1;
    }
}

fn obtener_objeto_con_header(
    tipo: u8,
    tamanio: u32,
    contenido_descomprimido: &mut Vec<u8>,
) -> Result<Vec<u8>, String> {
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
            return Err("Tipo de objeto invalido".to_string());
        }
    }
    header.append(contenido_descomprimido);
    Ok(header)
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

pub fn leer_packfile_y_escribir(
    bytes: &Vec<u8>,
    ubicacion: String,
) -> Result<(), String> {
    let checksum = verificar_checksum(bytes);
    match checksum {
        true => println!("Checksum correcto"),
        false => println!("Checksum incorrecto"),
    }
    let mut offset = 0;
    let firma = &bytes[offset..offset + 4];
    offset += 4;
    let version = &bytes[offset..offset + 4];
    offset += 4;
    let largo = &bytes[offset..offset + 4];
    offset += 4;
    let largo = u32::from_be_bytes([largo[0], largo[1], largo[2], largo[3]]);
    let mut contador: u32 = 0;

    while contador < largo {
        let offset_previo = offset.clone();
        let (mut tipo, mut tamanio) = decodificar_bytes_sin_borrado(bytes, &mut offset);
        let mut obj_data = vec![0; tamanio as usize];
        if tipo == 7 {
            let hash_obj = Tree::encode_hex(&bytes[offset..offset + 20]);
            offset += 20;
            // println!("Objeto base (hash): {:?}", &hash_obj);

            let mut objeto_descomprimido = vec![0; tamanio as usize];

            let mut descompresor = Decompress::new(true);

            descompresor
                .decompress(
                    &bytes[offset..],
                    &mut objeto_descomprimido,
                    FlushDecompress::None,
                )
                .map_err(|e| e.to_string())?;     

            let total_in = descompresor.total_in(); // cantidad de bytes en instrucciones
            offset += total_in as usize;

            // bytes.drain(0..total_in as usize);
            let mut _offset = 0;
            let tamanio_objeto_base = leer_varint(&mut objeto_descomprimido);
            // println!("tamanio objeto base: {:?}", tamanio_objeto_base);
            let tamanio_objeto_reconstruido = leer_varint(&mut objeto_descomprimido);
            // println!("tamanio objeto reconstruido: {:?}", tamanio_objeto_reconstruido);

            let byte_codificado: u8 = objeto_descomprimido[0];
            objeto_descomprimido.drain(0..1);
            // // copy indica con el offset de donde se parte a copiar los bytes y el size indica la cantidad de bytes a copiar
            // // luego pueden venir mas operaciones como append (al copy)
            // // ej: copiar un archivo de punta a punta y agregarle un ! seria offset = 0, size = len(), data instruction con !
            match byte_codificado & 0x80 {
                0 => {}
                1 => {}
                _ => {
                    println!("Error, no se reconoce la instruccion de reconstruccion");
                }
            }
            contador += 1;
            continue;
        }
        if tipo == 6 {
            (tipo, obj_data) = _read_ofs_delta_obj(bytes, tamanio, &mut offset, offset_previo)?;
            tamanio = obj_data.len() as u32;
        } else {
            // obj_data = vec![0; tamanio as usize];
            let mut descompresor = Decompress::new(true);

            descompresor
                .decompress(&bytes[offset..], &mut obj_data, FlushDecompress::None)
                .map_err(|e| e.to_string())?;
            let total_in = descompresor.total_in(); // esto es para calcular el offset
            offset += total_in as usize;
            // calculo el hash
        }
        let objeto = obtener_objeto_con_header(tipo, tamanio, &mut obj_data)?;
        let mut hasher = Sha1::new();
        hasher.update(objeto.clone());
        let _hash = hasher.finalize();
        let hash = format!("{:x}", _hash);
        let ruta = format!("{}{}/{}", &ubicacion, &hash[..2], &hash[2..]);
        io::escribir_bytes(ruta, compresion::comprimir_contenido_u8(&objeto)?)?;
        contador += 1;
    }
    Ok(())
}

fn read_vli_be(bytes: &Vec<u8>, actual_offset: &mut usize, offset: bool) -> usize {
    //     """Read a variable-length integer (big-endian)."""
    let mut val: usize = 0;
    loop {
        //         # add in the next 7 bits of data
        let byt = &bytes[*actual_offset];
        *actual_offset += 1;
        val = (val << 7) | (byt & 0x7f) as usize;
        if byt & 0x80 == 0 {
            // # nb: that was the last byte
            break;
        }
        if offset {
            val += 1
        }
    }
    val
}

fn _read_pack_object(bytes: &Vec<u8>, offset: &mut usize) -> Result<(u8, Vec<u8>), String> {
    let offset_pre_varint = offset.clone();
    let (tipo, tamanio) = decodificar_bytes_sin_borrado(bytes, offset);
    if tipo == 6 {
        _read_ofs_delta_obj(bytes, tamanio, offset, offset_pre_varint)
    } else {
        let mut objeto_descomprimido = vec![0; tamanio as usize];

        let mut descompresor = Decompress::new(true);

        descompresor
            .decompress(
                &bytes[*offset..],
                &mut objeto_descomprimido,
                FlushDecompress::None,
            )
            .map_err(|e| e.to_string());

        *offset += descompresor.total_in() as usize;
        Ok((tipo, objeto_descomprimido))
    }
}

fn _read_ofs_delta_obj(
    bytes: &Vec<u8>,
    obj_size: u32,
    actual_offset: &mut usize,
    offset_pre_varint: usize,
) -> Result<(u8, Vec<u8>), String> {
    let offset = read_vli_be(bytes, actual_offset, true);

    let base_obj_offset = offset_pre_varint - offset;

    let (base_obj_type, mut base_obj_data) =
        _read_pack_object(bytes, &mut (base_obj_offset as usize))?;

    _make_delta_obj(
        bytes,
        actual_offset,
        base_obj_type,
        &mut base_obj_data,
        obj_size,
    )
}

// Function that converts a slice of bytes to a usize, handling errors
fn bytes_to_usize(bytes: &[u8]) -> Result<usize, String> {
    bytes.try_into().map(usize::from_le_bytes).map_err(|_| "Fallo a convertir bytes a usize en packfile".to_string())
}

// Function that converts a slice of bytes to a usize, handling errors
fn bytes_to_u16(bytes: &[u8]) -> Result<u16, String> {
    bytes.try_into().map(u16::from_le_bytes).map_err(|_| "Fallo al convertir bytes a u32 en packfile".to_string())
}

fn _make_delta_obj(
    bytes: &Vec<u8>,
    actual_offset: &mut usize,
    base_obj_type: u8,
    base_obj_data: &mut Vec<u8>,
    obj_size: u32,
) -> Result<(u8, Vec<u8>), String> {
    let mut objeto_descomprimido = vec![0; obj_size as usize];

    let mut descompresor = Decompress::new(true);

    descompresor
        .decompress(
            &bytes[*actual_offset..],
            &mut objeto_descomprimido,
            FlushDecompress::None,
        )
        .map_err(|e| e.to_string());
    *actual_offset += descompresor.total_in() as usize;

    let mut data_descomprimida_offset: usize = 0;
    let base_obj_size = read_varint_le(&objeto_descomprimido, &mut data_descomprimida_offset);
    let obj_size2 = read_varint_le(&objeto_descomprimido, &mut data_descomprimida_offset);

    let mut obj_data: Vec<u8> = Vec::new();

    while data_descomprimida_offset < objeto_descomprimido.len() {
        let byt = &objeto_descomprimido[data_descomprimida_offset];
        data_descomprimida_offset += 1;
        if *byt == 0x00 {
            continue;
        }
        if (byt & 0x80) != 0 {
            let mut vals: Vec<u8> = Vec::new();
            for i in 0..6 + 1 {
                let bmask = 1 << i;
                if (byt & bmask) != 0 {
                    vals.push(objeto_descomprimido[data_descomprimida_offset]);
                    data_descomprimida_offset += 1;
                } else {
                    vals.push(0);
                }
            }
            // let start: usize = u32::from_le_bytes(vals[0..4].try_into().unwrap()) as usize;
            let start = bytes_to_usize(&vals[0..4])?;
            // let mut nbytes: usize = u16::from_le_bytes(vals[4..6].try_into().unwrap()) as usize;
            let mut nbytes: usize = bytes_to_u16(&vals[4..6])? as usize;
            if nbytes == 0 {
                nbytes = 0x10000
            }

            obj_data.extend(&base_obj_data[start..start + nbytes]);
        } else {
            let nbytes = byt & 0x7f;
            obj_data.extend(
                &objeto_descomprimido
                    [data_descomprimida_offset..data_descomprimida_offset + nbytes as usize],
            );
            data_descomprimida_offset += nbytes as usize;
        }
    }
    Ok((base_obj_type, obj_data))
}

fn read_varint_le(input: &Vec<u8>, offset: &mut usize) -> u32 {
    let mut result = 0u32;
    let mut shift = 0;

    loop {
        let byte = input[*offset];
        result |= ((byte & 0x7F) as u32) << shift;
        shift += 7;
        *offset += 1;

        if byte & 0x80 == 0 {
            break;
        }
    }
    result
}

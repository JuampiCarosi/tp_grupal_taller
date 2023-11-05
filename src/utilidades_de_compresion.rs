use crate::{io, tipos_de_dato::objetos::tree::Tree, utilidades_de_compresion};
use flate2::{read::ZlibDecoder, write::ZlibEncoder, Compression};
use std::io::{Read, Write};

// CAMBIAR GIT POR GIR

pub fn descomprimir_objeto(hash: String, ruta: String) -> Result<String, String> {
    let ruta_objeto = format!("{}{}/{}", ruta.clone(), &hash[..2], &hash[2..]);
    // println!("gonna decompress {} from: {}", hash,ruta_objeto);

    let contenido_leido = io::leer_bytes(ruta_objeto)?;
    let contenido_descomprimido = descomprimir_contenido_u8(&contenido_leido)?;
    let contenido_decodificado = decodificar_contenido(contenido_descomprimido)?;
    Ok(contenido_decodificado)
}

pub fn decodificar_contenido(contenido: Vec<u8>) -> Result<String, String> {
    let header_u8: &[u8] = contenido.split(|&x| x == 0).collect::<Vec<&[u8]>>()[0];

    let header = String::from_utf8(header_u8.to_vec()).unwrap();
    let tipo_objeto = header.split_whitespace().collect::<Vec<&str>>()[0];

    if tipo_objeto == "blob" {
        return Ok(String::from_utf8(contenido.clone()).unwrap());
    } else if tipo_objeto == "commit" {
        return Ok(String::from_utf8(contenido.clone()).unwrap());
    } else if tipo_objeto == "tree" {
        let mut spliteado_por_null: Vec<Vec<u8>> = Vec::new();
        let mut buffer: Vec<u8> = Vec::new();
        let mut i = 0;

        for char in contenido.iter() {
            if *char == 0 && (buffer.len() >= 20 || i < 2) {
                spliteado_por_null.push(buffer.clone());
                buffer.clear();
                i += 1;
            } else {
                buffer.push(*char);
            }
        }
        spliteado_por_null.push(buffer);

        let mut spliteado_por_null_separado_por_linea: Vec<Vec<u8>> = Vec::new();
        spliteado_por_null_separado_por_linea.push(spliteado_por_null[0].clone()); // tree
        spliteado_por_null_separado_por_linea.push(spliteado_por_null[1].clone()); // size

        let last_line = spliteado_por_null.pop(); // saco ultima que es hash

        spliteado_por_null.iter().skip(2).for_each(|x| {
            let (hash, modo_y_nombre) = x.split_at(20);
            spliteado_por_null_separado_por_linea.push(hash.to_vec());
            spliteado_por_null_separado_por_linea.push(modo_y_nombre.to_vec());
        });

        spliteado_por_null_separado_por_linea.push(last_line.unwrap().clone());

        let mut contenido = format!("{}\0", header);

        for i in (0..(spliteado_por_null_separado_por_linea.len()))
            .skip(1)
            .step_by(2)
        {
            if i + 1 < spliteado_por_null_separado_por_linea.len() {
                let modo_y_nombre =
                    String::from_utf8(spliteado_por_null_separado_por_linea[i].clone()).unwrap();
                let hash = Tree::encode_hex(&spliteado_por_null_separado_por_linea[i + 1]);

                let linea = format!("{modo_y_nombre}\0{hash}");
                contenido.push_str(&linea);
            } else {
                return Err("Error al decodificar el contenido del tree".to_string());
            }
        }

        return Ok(contenido);
    }

    Err("Tipo de objeto invalido".to_string())
}

pub fn comprimir_contenido(contenido: String) -> Result<Vec<u8>, String> {
    let mut compresor = ZlibEncoder::new(Vec::new(), Compression::default());
    if compresor.write_all(contenido.as_bytes()).is_err() {
        return Err("No se pudo comprimir el contenido".to_string());
    };
    match compresor.finish() {
        Ok(contenido_comprimido) => Ok(contenido_comprimido),
        Err(_) => Err("No se pudo comprimir el contenido".to_string()),
    }
}

pub fn comprimir_contenido_u8(contenido: &Vec<u8>) -> Result<Vec<u8>, String> {
    let mut compresor = ZlibEncoder::new(Vec::new(), Compression::default());
    if compresor.write_all(contenido).is_err() {
        return Err("No se pudo comprimir el contenido".to_string());
    };
    match compresor.finish() {
        Ok(contenido_comprimido) => Ok(contenido_comprimido),
        Err(_) => Err("No se pudo comprimir el contenido".to_string()),
    }
}

pub fn descomprimir_contenido_u8(contenido: &[u8]) -> Result<Vec<u8>, String> {
    let mut descompresor = ZlibDecoder::new(contenido);
    let mut contenido_descomprimido = Vec::new();
    match descompresor.read_to_end(&mut contenido_descomprimido) {
        Ok(_) => {}
        Err(_) => Err("No se pudo descomprimir el contenido")?,
    };
    Ok(contenido_descomprimido)
}

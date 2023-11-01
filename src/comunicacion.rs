use crate::err_comunicacion::ErrorDeComunicacion;
use crate::io;
use crate::packfile;
use crate::tipos_de_dato::comandos::hash_object::HashObject;
use crate::tipos_de_dato::logger;
use crate::utilidades_de_compresion::comprimir_contenido;
use crate::utilidades_de_compresion::decodificar_contenido;
use flate2::{Decompress, FlushDecompress};
use std::convert::TryInto;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::str;

use sha1::{Digest, Sha1};

pub struct Comunicacion {
    flujo: TcpStream,
}

impl Comunicacion {
    ///Inicia el flujo de comunicacion con la direccion de servidor recivida.
    ///Pensanda para ser usada desde el cliente
    /// 
    /// Argumentos:
    /// direccion_servidor: la direccion del servidor con la cual se desea iniciar
    ///     la coneccion. Tiene el formato 'ip:puerto'
    ///
    /// Errores:
    /// -Puede fallar el flujo
    pub fn new_desde_direccion_servidor(direccion_servidor: &str) -> Result<Comunicacion, String> {
        let flujo = TcpStream::connect(direccion_servidor)
            .map_err(|e| format!("Fallo en en la conecciion con el servido.\n{}\n", e))?;

        Ok(Comunicacion { flujo })
    }

    pub fn new(flujo:TcpStream)->Comunicacion{
        Comunicacion {flujo}
    }

    pub fn enviar(&mut self, mensaje: &str) -> Result<(), String> {
        self.flujo
            .write_all(mensaje.as_bytes())
            .map_err(|e| format!("Fallo en el envio del mensaje.\n{}\n", e))
    }

    pub fn aceptar_pedido(&mut self) -> Result<String, ErrorDeComunicacion> {
        // lee primera parte, 4 bytes en hexadecimal indican el largo del stream
        let mut tamanio_bytes = [0; 4];
        self.flujo.read_exact(&mut tamanio_bytes)?;
        // largo de bytes a str
        let tamanio_str = str::from_utf8(&tamanio_bytes)?;
        // transforma str a u32
        let tamanio = u32::from_str_radix(tamanio_str, 16).unwrap();
        // lee el resto del flujo
        let mut data = vec![0; (tamanio - 4) as usize];
        self.flujo.read_exact(&mut data)?;
        let linea = str::from_utf8(&data)?;
        Ok(linea.to_string())
    }
    // obtiene lineas en formato PKT
    pub fn obtener_lineas(&mut self) -> Result<Vec<String>, ErrorDeComunicacion> {
        let mut lineas: Vec<String> = Vec::new();
        loop {
            // lee primera parte, 4 bytes en hexadecimal indican el largo del stream
            let mut tamanio_bytes = [0; 4];
            self.flujo.read_exact(&mut tamanio_bytes)?;
            // largo de bytes a str
            let tamanio_str = str::from_utf8(&tamanio_bytes)?;
            // transforma str a u32
            let tamanio = u32::from_str_radix(tamanio_str, 16).unwrap();
            if tamanio == 0 {
                break;
            }
            // lee el resto del flujo
            let mut data = vec![0; (tamanio - 4) as usize];
            self.flujo.read_exact(&mut data)?;
            let linea = str::from_utf8(&data)?;
            lineas.push(linea.to_string());
            if linea.contains("NAK") || linea.contains("done") {
                break;
            }
        }
        println!("Received: {:?}", lineas);
        Ok(lineas)
    }

    pub fn responder(&mut self, lineas: Vec<String>) -> Result<(), ErrorDeComunicacion> {
        for linea in &lineas {
            self.flujo.write_all(linea.as_bytes())?;
        }
        if !lineas[0].contains(&"NAK".to_string())
            && !lineas[0].contains(&"ACK".to_string())
            && !lineas[0].contains(&"done".to_string())
        {
            self.flujo.write_all(String::from("0000").as_bytes())?;
        }
        Ok(())
    }

    pub fn responder_con_bytes(&mut self, lineas: Vec<u8>) -> Result<(), ErrorDeComunicacion> {
        self.flujo.write_all(&lineas)?;
        if !lineas.starts_with(b"PACK") {
            self.flujo.write_all(String::from("0000").as_bytes())?;
        }
        Ok(())
    }

    pub fn obtener_lineas_como_bytes(&mut self) -> Result<Vec<u8>, ErrorDeComunicacion> {
        let mut buffer = Vec::new();
        let mut temp_buffer = [0u8; 1024]; // Tamaño del búfer de lectura

        loop {
            let bytes_read = self.flujo.read(&mut temp_buffer)?;

            if bytes_read == 0 {
                break; // No hay más bytes disponibles, salir del bucle
            }
            // Copiar los bytes leídos al búfer principal
            buffer.extend_from_slice(&temp_buffer[0..bytes_read]);
        }
        Ok(buffer)
    }

    pub fn obtener_obj_ids(&mut self, lineas: &Vec<String>) -> Vec<String> {
        println!("lineas: {:?}", lineas);
        let mut obj_ids: Vec<String> = Vec::new();
        for linea in lineas {
            obj_ids.push(linea.split_whitespace().collect::<Vec<&str>>()[0].to_string());
        }
        obj_ids
    }

    pub fn obtener_wants_pkt(
        &mut self,
        lineas: &Vec<String>,
        capacidades: String,
    ) -> Result<Vec<String>, ErrorDeComunicacion> {
        // hay que checkear que no haya repetidos, usar hashset
        let mut lista_wants: Vec<String> = Vec::new();
        let mut obj_ids = self.obtener_obj_ids(lineas);
        println!("obj_ids: {:?}", obj_ids);
        obj_ids[0].push_str(&(" ".to_string() + &capacidades)); // le aniado las capacidades
        for linea in obj_ids {
            lista_wants.push(io::obtener_linea_con_largo_hex(
                &("want ".to_string() + &linea + "\n"),
            ));
        }
        Ok(lista_wants)
    }
    pub fn obtener_haves_pkt(&mut self, lineas: &Vec<String>) -> Vec<String> {
        let mut haves: Vec<String> = Vec::new();
        for linea in lineas {
            haves.push(io::obtener_linea_con_largo_hex(
                &("have ".to_string() + &linea + "\n"),
            ))
        }
        haves
    }

    pub fn obtener_paquete_y_escribir(
        &mut self,
        bytes: &mut Vec<u8>,
        ubicacion: String,
    ) -> Result<(), ErrorDeComunicacion> {
        // a partir de aca obtengo el paquete
        // println!("cant bytes: {:?}", bytes.len());
        // println!("obteniendo firma");
        let firma = &bytes[0..4];
        println!("firma: {:?}", str::from_utf8(&firma));
        // assert_eq!("PACK", str::from_utf8(&firma).unwrap());
        bytes.drain(0..4);
        let version = &bytes[0..4];
        println!("version: {:?}", str::from_utf8(&version)?);
        // assert_eq!("0002", str::from_utf8(&version)?);

        bytes.drain(0..4);
        // println!("obteniendo largo");
        let largo = &bytes[0..4];
        let largo_packfile: [u8; 4] = largo.try_into().unwrap();
        let largo = u32::from_be_bytes(largo_packfile);
        println!("largo: {:?}", largo);
        bytes.drain(0..4);

        while bytes.len() > 0 {
            // println!("cant bytes: {:?}", bytes.len());
            let (tipo, tamanio, bytes_leidos) = packfile::decodificar_bytes(bytes);
            println!("tipo: {:?}, tamanio: {}", tipo, tamanio);
            // println!("cant bytes post decodificacion: {:?}", bytes.len());
            // println!("tipo: {:?}", tipo);
            // println!("tamanio: {:?}", tamanio);

            // // -- leo el contenido comprimido --
            let mut objeto_descomprimido = vec![0; tamanio as usize];

            let mut descompresor = Decompress::new(true);

            descompresor
                .decompress(&bytes, &mut objeto_descomprimido, FlushDecompress::None)
                .unwrap();

            let mut hasher = Sha1::new();
            hasher.update(objeto_descomprimido.clone());
            let _hash = hasher.finalize();
            let hash = format!("{:x}", _hash);

            println!("hash: {:?}", hash);
            let ruta = format!("{}{}/{}", &ubicacion, &hash[..2], &hash[2..]);
            println!("rutarda donde pongo objetos: {:?}", ruta);

            let total_out = descompresor.total_out(); // esto es lo que debe matchear el tamanio que se pasa en el header
            let total_in = descompresor.total_in(); // esto es para calcular el offset
            println!(
                "total in: {:?}, total out: {:?} ",
                total_in as usize, total_out as usize
            );

            io::escribir_bytes(ruta, bytes.drain(0..total_in as usize)).unwrap();

            // println!("cant bytes restantes: {:?}", bytes.len());
        }
        Ok(())
    }
}

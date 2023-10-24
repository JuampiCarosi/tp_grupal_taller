use crate::err_comunicacion::ErrorDeComunicacion;
use crate::tipos_de_dato::logger;
use std::net::TcpStream;
use std::io::{Read, Write};
use std::str;
use crate::io;
use crate::tipos_de_dato::comandos::cat_file::CatFile;
use std::rc::Rc;

pub struct Comunicacion {
    flujo: TcpStream,
}

impl Comunicacion {
    pub fn new(flujo: TcpStream) -> Comunicacion {
        Comunicacion {flujo}
    }

    pub fn aceptar_pedido(&mut self) -> Result<String, ErrorDeComunicacion>{
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
            // println!("Received: {:?}", linea);
            lineas.push(linea.to_string());
        }
        // println!("Received: {:?}", lineas);
        Ok(lineas)
    }
    
    pub fn responder(&mut self, lineas: Vec<String>) -> Result<(), ErrorDeComunicacion> {
        for linea in &lineas { 
            self.flujo.write_all(linea.as_bytes())?;
        }
        // if lineas[0][4..] != "NAK".to_string() {
            self.flujo.write_all(String::from("0000").as_bytes())?;
        // }
        Ok(())
    }
    
    pub fn responder_con_bytes(&mut self, lineas: Vec<u8>) -> Result<(), ErrorDeComunicacion> {
        self.flujo.write_all(&lineas)?;
        self.flujo.write_all(String::from("0000").as_bytes())?;
        Ok(())
    }
    
    pub fn obtener_obj_ids(&mut self, lineas: &Vec<String>) -> Vec<String> {
        let mut obj_ids: Vec<String> = Vec::new();
        for linea in lineas {
            obj_ids.push(linea.split_whitespace().collect::<Vec<&str>>()[0].to_string());
        }
        obj_ids
    }
    
    
    pub fn obtener_wants(&mut self, lineas: &Vec<String>, capacidades: String) -> Result<Vec<String>, ErrorDeComunicacion> {
        // hay que checkear que no haya repetidos, usar hashset
        let mut lista_wants: Vec<String> = Vec::new();
        let mut obj_ids = self.obtener_obj_ids(lineas);
        obj_ids[0].push_str(&(" ".to_string() + &capacidades));
        for linea in obj_ids {
            lista_wants.push(io::obtener_linea_con_largo_hex(&("want".to_string() + &linea)));     
        }
        Ok(lista_wants)
    }
    
    pub fn obtener_paquete(&mut self) -> Result<(), ErrorDeComunicacion> {
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
        println!("Received: {:?}", linea);
        println!("Borrar esto de obtener_paquete");
        self.flujo.read_exact(&mut tamanio_bytes)?;
        println!("fin de lo random  ");


        // a partir de aca obtengo el paquete
        // print!("obteniendo firma");
        // let mut firma = [0; 4];
        // self.flujo.read_exact(&mut firma)?;
        // println!("firma: {:?}", str::from_utf8(&firma));
        // assert_eq!("PACK", str::from_utf8(&firma).unwrap());
        
        // let mut version = [0; 4];
        // self.flujo.read_exact(&mut version)?;
        // println!("version: {:?}", str::from_utf8(&version)?);
        // assert_eq!("0002", str::from_utf8(&version)?);
    
        // println!("obteniendo paquete");
        // let mut largo = [0; 4];
        // self.flujo.read_exact(&mut largo)?;
        // let _largo = u32::from_be_bytes(largo);
        

        // let n_byte: u8 = 0;
        // self.flujo.read_exact(&mut [n_byte])?;
        
        // let mut bytes_obj = [0; 20];
        // self.flujo.read_exact(&mut bytes_obj)?;
        // let hash = str::from_utf8(&bytes_obj)?;
        // let logger = Rc::new(logger::Logger::new().unwrap());
    
        // let _cat_file = CatFile::from(&mut vec![hash.to_string(), "-p".to_string()], logger.clone()).unwrap().ejecutar();
        Ok(())
    }
}   


// pub fn obtener_capacidades(referencias: Vec<String>) -> Vec<&'static str> {
//     let capacidades = referencias[0].split("\0").collect::<Vec<&str>>().clone();
    
// }

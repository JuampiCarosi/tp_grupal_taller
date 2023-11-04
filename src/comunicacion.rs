use crate::err_comunicacion::ErrorDeComunicacion;
use crate::io;
use crate::packfile;
use flate2::{Decompress, FlushDecompress};
use std::convert::TryInto;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::str;
use std::str::Bytes;
use std::sync::Mutex;

use sha1::{Digest, Sha1};

pub struct Comunicacion<T: Read + Write> {
    flujo: Mutex<T>,
}

impl<T: Write + Read> Comunicacion<T> {
    ///Inicia el flujo de comunicacion con la direccion de servidor recivida.
    ///Pensanda para ser usada desde el cliente
    ///
    /// Argumentos:
    /// direccion_servidor: la direccion del servidor con la cual se desea iniciar
    ///     la coneccion. Tiene el formato 'ip:puerto'
    ///
    /// Errores:
    /// -Puede fallar el flujo
    pub fn new_desde_direccion_servidor(
        direccion_servidor: &str,
    ) -> Result<Comunicacion<TcpStream>, String> {
        let flujo = Mutex::new(
            TcpStream::connect(direccion_servidor)
                .map_err(|e| format!("Fallo en en la conecciion con el servido.\n{}\n", e))?,
        );

        Ok(Comunicacion { flujo })
    }

    pub fn new(flujo: T) -> Comunicacion<T> {
        Comunicacion { flujo: Mutex::new(flujo) }
    }

    pub fn enviar(&self, mensaje: &str) -> Result<(), String> {
        self.flujo
            .lock()
            .map_err(|e| format!("Fallo en el envio del mensaje.\n{}\n", e))?
            .write_all(mensaje.as_bytes())
            .map_err(|e| format!("Fallo en el envio del mensaje.\n{}\n", e))
    }

    pub fn aceptar_pedido(&self) -> Result<String, ErrorDeComunicacion> {
        // lee primera parte, 4 bytes en hexadecimal indican el largo del stream
        let mut tamanio_bytes = [0; 4];
        self.flujo.lock().unwrap().read_exact(&mut tamanio_bytes)?;
        // largo de bytes a str
        let tamanio_str = str::from_utf8(&tamanio_bytes)?;
        // transforma str a u32
        let tamanio = u32::from_str_radix(tamanio_str, 16).unwrap();
        if tamanio == 0{
            return Ok('\0'.to_string());
        }
        // lee el resto del flujo
        let mut data = vec![0; (tamanio - 4) as usize];
        self.flujo.lock().unwrap().read_exact(&mut data)?;
        let linea = str::from_utf8(&data)?;
        Ok(linea.to_string())
    }

    fn leer_del_flujo_tantos_bytes(
        &self,
        cantida_bytes_a_leer: usize,
    ) -> Result<Vec<u8>, String> {
        let mut data = vec![0; cantida_bytes_a_leer];
        self.flujo.lock()
        .map_err(|e| format!("Fallo en el mutex de la lectura.\n{}\n", e))?.read_exact(&mut data).map_err(|e| {
            format!(
                "Fallo en obtener la linea al leer los priemeros 4 bytes.\n{}\n",
                e
            )
        })?;
        Ok(data)
    }

    fn leer_del_flujo_tantos_bytes_en_string(
        &self,
        cantida_bytes_a_leer: usize,
    ) -> Result<String, String> {
        let data = self.leer_del_flujo_tantos_bytes(cantida_bytes_a_leer)?;
        let contendio = str::from_utf8(&data).map_err(|e| {
            format!(
                "Fallo en castear el contenido a String en leer del flujo.\n{}\n",
                e
            )
        })?;

        Ok(contendio.to_string())
    }

    ///lee la primera parte de la linea actual, obtiene el largo de la linea actual
    ///
    /// # Resultado:
    /// -Devuelve el largo de la linea actual (ojo !!contando todavia los primeros 4 bytes
    /// de la linea donde dice el largo) en u32
    fn obtener_largo_de_la_linea(&self) -> Result<u32, String> {
        let bytes_tamanio_linea = 4;
        let tamanio = self.leer_del_flujo_tantos_bytes(bytes_tamanio_linea)?;
        let tamanio_u32 = u32::from_le_bytes([tamanio[0], tamanio[1], tamanio[2], tamanio[3]]);

        Ok(tamanio_u32)
    }

    /// lee el contendio de la linea actual, es decir, lee el resto del flujo que no incluye el
    /// largo
    ///
    /// # Argumentos
    /// - tamanio: Es el largo de la linea actual(contando los primeros 4 bytes del largo y el contedio)
    ///
    /// # Resultado
    /// - Devuelve el contendio de la linea actual
    fn obtener_contenido_linea(&self, tamanio: u32) -> Result<String, String> {
        let tamanio_sin_largo = (tamanio - 4) as usize;
        let linea = self.leer_del_flujo_tantos_bytes_en_string(tamanio_sin_largo)?;
        Ok(linea)
    }

    /// Obtiene todo el contenido envio por el servidor hasta un NAK o done( sacando
    /// los bytes referentes al contendio),obtiene lineas en formato PKT.
    ///
    /// # Resultado
    /// - Devuelve cada linea envia por el servidor (sin el largo)
    ///
    pub fn obtener_lineas(&self) -> Result<Vec<String>, String> {
        let mut lineas: Vec<String> = Vec::new();
        loop {
            // let tamanio = {let mut tamanio_bytes = [0; 4];
            //     self.flujo.lock().unwrap().read_exact(&mut tamanio_bytes).unwrap();
            // tamanio_bytes};
            // if tamanio.is_empty(){
            //     break;
            // }

            // let tamanio_str = str::from_utf8(&tamanio).unwrap();
            // // transforma str a u32
            // let tamanio = u32::from_str_radix(tamanio_str, 16).unwrap();

            // let linea = self.obtener_contenido_linea(tamanio)?;
            let linea = self.aceptar_pedido().unwrap();
            if linea == '\0'.to_string(){
                break;
            }
            lineas.push(linea.to_string());

            if linea.contains("NAK") || linea.contains("done") {
                break;
            }
        }
        Ok(lineas)
    }

    pub fn responder(&self, lineas: Vec<String>) -> Result<(), ErrorDeComunicacion> {
        for linea in &lineas {
            self.flujo.lock().unwrap().write_all(linea.as_bytes())?;
        }
        if !lineas[0].contains(&"NAK".to_string())
            && !lineas[0].contains(&"ACK".to_string())
            && !lineas[0].contains(&"done".to_string())
        {
            self.flujo.lock().unwrap().write_all(String::from("0000").as_bytes())?;
        }
        Ok(())
    }

    pub fn responder_con_bytes(&self, lineas: Vec<u8>) -> Result<(), ErrorDeComunicacion> {
        self.flujo.lock().unwrap().write_all(&lineas)?;
        if !lineas.starts_with(b"PACK") {
            self.flujo.lock().unwrap().write_all(String::from("0000").as_bytes())?;
        }
        Ok(())
    }

    pub fn obtener_lineas_como_bytes(&self) -> Result<Vec<u8>, String> {
        let mut buffer = Vec::new();
        let mut temp_buffer = [0u8; 1024]; // Tamaño del búfer de lectura

        loop {
            let bytes_read = self.flujo.lock()
            .map_err(|e| format!("Fallo en el mutex.\n{}\n", e))?.read(&mut temp_buffer).map_err(|e| {
                format!("Fallo en la lectura de la respuesta del servidor.\n{}\n", e)
            })?;

            if bytes_read == 0 {
                break; // No hay más bytes disponibles, salir del bucle
            }
            // Copiar los bytes leídos al búfer principal
            buffer.extend_from_slice(&temp_buffer[0..bytes_read]);
        }
        Ok(buffer)
    }

    pub fn obtener_obj_ids(&self, lineas: &Vec<String>) -> Vec<String> {
        println!("lineas: {:?}", lineas);
        let mut obj_ids: Vec<String> = Vec::new();
        for linea in lineas {
            obj_ids.push(linea.split_whitespace().collect::<Vec<&str>>()[0].to_string());
        }
        obj_ids
    }

    pub fn obtener_wants_pkt(
        &self,
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

    ///Envia al servidor todos los pedidos en el formato correspondiente, junto con las capacidades para la
    /// comunicacion y finaliza con un '0000' de ser necesario.
    ///
    /// El objetivo es enviar las commits pertenecientes a las cabeza de rama que es necesario actualizar
    /// El formato seguido para cada linea es la siguiente:
    /// - ''4 bytes con el largo'want 'hash de un commit cabeza de rama'  
    /// Y la primera linea, ademas contien las capacidades separadas por espacio
    ///
    /// # Argumentos
    ///
    /// - pedidos: lista con los hash de los commits cabeza de rama de las ramas que se quieren actualizar en local
    ///     con respecto a las del servidor
    /// - capacidades: las capacidades que va a ver en la comunicacion con el servidor  
    pub fn enviar_pedidos_al_servidor_pkt(
        &self,
        mut pedidos: Vec<String>,
        capacidades: String,
    ) -> Result<(), String> {
        self.anadir_capacidades_primer_pedido(&mut pedidos, capacidades);

        for pedido in &mut pedidos {
            let pedido_con_formato = self.dar_formato_de_solicitud(pedido);
            self.enviar(&pedido_con_formato)?;
        }

        if !pedidos[0].contains(&"NAK".to_string())
            && !pedidos[0].contains(&"ACK".to_string())
            && !pedidos[0].contains(&"done".to_string())
        {
            self.enviar("0000")?;
        }
        Ok(())
    }

    ///Le añade las capadcidades al primer objeto para cumplir con el protocolo
    fn anadir_capacidades_primer_pedido(&self, pedidos: &mut Vec<String>, capacidades: String) {
        pedidos[0].push_str(&(" ".to_string() + &capacidades));
    }
    ///recibi el hash de un commit y le da el formato correcto para hacer el want
    fn dar_formato_de_solicitud(&self, hash_commit: &mut String) -> String {
        io::obtener_linea_con_largo_hex(&("want ".to_string() + &hash_commit + "\n"))
    }

    pub fn obtener_haves_pkt(&self, lineas: &Vec<String>) -> Vec<String> {
        let mut haves: Vec<String> = Vec::new();
        for linea in lineas {
            haves.push(io::obtener_linea_con_largo_hex(
                &("have ".to_string() + &linea + "\n"),
            ))
        }
        haves
    }

    ///Envia al servidor todo el contendio(los hash de los objetos) que ya se tiene y que no debe
    /// mandarle
    ///
    /// # Argumentos
    ///
    /// - hash_objetos: lista con todos los hash de los objetos que ya se tiene y que no debe mandar
    ///     el servidor
    pub fn enviar_lo_que_tengo_al_servidor_pkt(
        &self,
        hash_objetos: &Vec<String>,
    ) -> Result<(), String> {
        for hash_objeto in hash_objetos {
            let pedido_con_formato = self.dar_formato_have(hash_objeto);
            self.enviar(&pedido_con_formato)?;
        }

        //esto if no se si va, el 0000 si
        if !hash_objetos[0].contains(&"NAK".to_string())
            && !hash_objetos[0].contains(&"ACK".to_string())
            && !hash_objetos[0].contains(&"done".to_string())
        {
            self.enviar("0000")?;
        }

        Ok(())
    }

    ///recibi el hash de un objeto y le da el formato correcto para hacer el have
    fn dar_formato_have(&self, hash_commit: &String) -> String {
        io::obtener_linea_con_largo_hex(&("have ".to_string() + &hash_commit + "\n"))
    }

    pub fn obtener_paquete_y_escribir(
        &self,
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

#[cfg(test)]
mod test {
    use std::{io::Read, io::Write, rc::Rc};

    use super::Comunicacion;

    struct MockTcpStream {
        lectura_data: Vec<u8>,
        escritura_data: Vec<u8>,
    }

    impl Read for MockTcpStream {
        fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
            let bytes_to_read = std::cmp::min(buf.len(), self.lectura_data.len());
            buf[..bytes_to_read].copy_from_slice(&self.lectura_data[..bytes_to_read]);
            self.lectura_data.drain(..bytes_to_read);
            Ok(bytes_to_read)
        }
    }

    impl Write for MockTcpStream {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            self.escritura_data.write(buf)
        }

        fn flush(&mut self) -> std::io::Result<()> {
            self.escritura_data.flush()
        }
    }
    #[test]

    fn test01_se_envia_mensajes_de_forma_correcta() {
        let mut mock = MockTcpStream {
            lectura_data: Vec::new(),
            escritura_data: Vec::new(),
        };

        Comunicacion::new(&mut mock).enviar("Hola server, soy siro. Todo bien ??").unwrap();

        assert_eq!(
            "Hola server, soy siro. Todo bien ??".as_bytes(),
            mock.escritura_data.as_slice()
        )
    }

    #[test]

    fn test02_se_obtiene_el_contenido_del_server_de_forma_correcta() {
        let contenido_mock = "000eversion 1 00887217a7c7e582c46cec22a130adf4b9d7d950fba0 HEAD\0multi_ack thin-pack side-band side-band-64k ofs-delta shallow no-progress include-tag 00441d3fcd5ced445d1abc402225c0b8a1299641f497 refs/heads/integration 003f7217a7c7e582c46cec22a130adf4b9d7d950fba0 refs/heads/master 003cb88d2441cac0977faf98efc80305012112238d9d refs/tags/v0.9 003c525128480b96c89e6418b1e40909bf6c5b2d580f refs/tags/v1.0 003fe92df48743b7bc7d26bcaabfddde0a1e20cae47c refs/tags/v1.0^{} 0000";
        
        let mut mock = MockTcpStream {
            lectura_data: contenido_mock.as_bytes().to_vec(),
            escritura_data: Vec::new(),
        };

        let lineas = Comunicacion::new(&mut mock).obtener_lineas().unwrap().join("\n");        

        let resultado_esperado_de_obtener_lineas = "version 1 \n\
        7217a7c7e582c46cec22a130adf4b9d7d950fba0 HEAD\0multi_ack thin-pack \
       side-band side-band-64k ofs-delta shallow no-progress include-tag \n\
        1d3fcd5ced445d1abc402225c0b8a1299641f497 refs/heads/integration \n\
        7217a7c7e582c46cec22a130adf4b9d7d950fba0 refs/heads/master \n\
        b88d2441cac0977faf98efc80305012112238d9d refs/tags/v0.9 \n\
        525128480b96c89e6418b1e40909bf6c5b2d580f refs/tags/v1.0 \n\
        e92df48743b7bc7d26bcaabfddde0a1e20cae47c refs/tags/v1.0^{} ";

        assert_eq!(
            resultado_esperado_de_obtener_lineas.to_string(),
            lineas
        )
    }


    
}

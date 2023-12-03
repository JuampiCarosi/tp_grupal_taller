use crate::err_comunicacion::ErrorDeComunicacion;
use crate::tipos_de_dato::packfile::Packfile;
use crate::utils::{self, io};
use std::io::{Read, Write};
use std::net::TcpStream;
use std::str;
use std::sync::{Arc, Mutex};

use super::logger::Logger;

pub enum RespuestaDePedido {
    Mensaje(String),
    Terminate,
}

pub struct Comunicacion<T: Read + Write> {
    flujo: Mutex<T>,
    repositorio: String,
    logger: Arc<Logger>,
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
        logger: Arc<Logger>,
    ) -> Result<Comunicacion<TcpStream>, String> {
        let partes: Vec<&str> = direccion_servidor.split('/').collect();
        let ip_puerto = partes[0];
        let repositorio = "/".to_string() + partes[1] + "/";
        let flujo = Mutex::new(
            TcpStream::connect(ip_puerto)
                .map_err(|e| format!("Fallo en en la conecciion con el servidor.\n{}\n", e))?,
        );
        // let repositorio = "/gir/".to_string();
        Ok(Comunicacion {
            flujo,
            repositorio,
            logger,
        })
    }

    ///Crea una comunicacion en base a una url.
    /// La url tiene el formato ip:puerto/repositorio/
    pub fn new_desde_url(
        url: &str,
        logger: Arc<Logger>,
    ) -> Result<Comunicacion<TcpStream>, String> {
        let (ip_puerto, repositorio) = utils::strings::obtener_ip_puerto_y_repositorio(url)?;

        let flujo = Mutex::new(
            TcpStream::connect(ip_puerto)
                .map_err(|e| format!("Fallo en en la coneccion con el servidor.\n{}\n", e))?,
        );

        Ok(Comunicacion {
            flujo,
            repositorio,
            logger,
        })
    }

    pub fn new_para_testing(flujo: T, logger: Arc<Logger>) -> Comunicacion<T> {
        let repositorio = "/gir/".to_string();

        Comunicacion {
            logger,
            flujo: Mutex::new(flujo),
            repositorio,
        }
    }

    pub fn new_desde_gir_config(logger: Arc<Logger>) -> Result<Comunicacion<TcpStream>, String> {
        let direccion_servidor = match utils::gir_config::conseguir_url_servidor() {
            Some(direccion_servidor) => direccion_servidor,
            None => {
                return Err("No se pudo obtener la direccion del servidor del config".to_string())
            }
        };
        Self::new_desde_direccion_servidor(&direccion_servidor, logger)
    }

    pub fn enviar(&self, mensaje: &str) -> Result<(), String> {
        self.enviar_bytes(mensaje.as_bytes())
    }

    pub fn enviar_bytes(&self, mensaje: &[u8]) -> Result<(), String> {
        self.flujo
            .lock()
            .map_err(|e| format!("Fallo en el envio del mensaje.\n{}\n", e))?
            .write_all(mensaje)
            .map_err(|e| format!("Fallo en el envio del mensaje.\n{}\n", e))
    }

    ///Inicia el comando git upload pack con el servidor, mandole al servidor el siguiente mensaje
    /// en formato:
    ///
    /// - ''git-upload-pack 'directorio'\0host='host'\0\0verision='numero de version'\0''
    ///
    pub fn iniciar_git_upload_pack_con_servidor(&self) -> Result<(), String> {
        self.logger.log("Iniciando git upload pack con el servidor");
        let comando = "git-upload-pack";
        let repositorio = &self.repositorio;
        let host = "gir.com";
        let numero_de_version = 1;

        let mensaje = format!(
            "{} {}\0host={}\0\0version={}\0",
            comando, repositorio, host, numero_de_version
        );
        let pedido = io::obtener_linea_con_largo_hex(&mensaje);
        self.enviar(&pedido)?;
        Ok(())
    }

    ///Inicia el comando git upload pack con el servidor, mandole al servidor el siguiente mensaje
    /// en formato:
    ///
    /// - ''git-upload-pack 'directorio'\0host='host'\0\0verision='numero de version'\0''
    ///
    pub fn iniciar_git_recive_pack_con_servidor(&self) -> Result<(), String> {
        self.logger
            .log("Iniciando git receive pack con el servidor");
        let comando = "git-receive-pack";
        let repositorio = &self.repositorio;
        let host = "gir.com";
        let numero_de_version = 1;

        let mensaje = format!(
            "{} {}\0host={}\0\0version={}\0",
            comando, repositorio, host, numero_de_version
        );
        let pedido = io::obtener_linea_con_largo_hex(&mensaje);
        self.enviar(&pedido)?;
        Ok(())
    }

    pub fn aceptar_pedido(&self) -> Result<RespuestaDePedido, String> {
        // lee primera parte, 4 bytes en hexadecimal indican el largo del stream

        let mut tamanio_bytes = [0; 4];
        self.flujo.lock().unwrap().read(&mut tamanio_bytes).map_err(|e| e.to_string())?;
        // largo de bytes a str
        if tamanio_bytes == [0, 0, 0, 0] {
            return Ok(RespuestaDePedido::Terminate);
        }

        let tamanio_str = str::from_utf8(&tamanio_bytes).map_err(|e| e.to_string())?;
        // transforma str a u32
        let tamanio = u32::from_str_radix(tamanio_str, 16).map_err(|e| e.to_string())?;
        if tamanio == 0 {
            return Ok(RespuestaDePedido::Mensaje('\0'.to_string()));
        }
        // lee el resto del flujo
        let mut data = vec![0; (tamanio - 4) as usize];
        self.flujo.lock().unwrap().read_exact(&mut data).map_err(|e| e.to_string())?;
        let linea = str::from_utf8(&data).map_err(|e| e.to_string())?;
        // if linea.contains("done") {
        // self.aceptar_pedido()?;
        // }
        Ok(RespuestaDePedido::Mensaje(linea.to_string()))
    }

    fn leer_del_flujo_tantos_bytes(&self, cantida_bytes_a_leer: usize) -> Result<Vec<u8>, String> {
        let mut data = vec![0; cantida_bytes_a_leer];
        self.flujo
            .lock()
            .map_err(|e| format!("Fallo en el mutex de la lectura.\n{}\n", e))?
            .read(&mut data)
            .map_err(|e| {
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
        let contenido = str::from_utf8(&data).map_err(|e| {
            format!(
                "Fallo en castear el contenido a String en leer del flujo.\n{}\n",
                e
            )
        })?;
        Ok(contenido.to_string())
    }

    ///lee la primera parte de la linea actual, obtiene el largo de la linea actual
    ///
    /// # Resultado:
    /// -Devuelve el largo de la linea actual (ojo !!contando todavia los primeros 4 bytes
    /// de la linea donde dice el largo) en u32
    fn obtener_largo_de_la_linea(&self) -> Result<u32, String> {
        let bytes_tamanio_linea = 4;
        let tamanio_str = self.leer_del_flujo_tantos_bytes_en_string(bytes_tamanio_linea)?;
        let tamanio_u32 = u32::from_str_radix(&tamanio_str, 16)
            .map_err(|e| format!("Fallo en la conversion a entero\n{}\n", e))?;
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
            let tamanio = self.obtener_largo_de_la_linea()?;
            if tamanio == 0 {
                break;
            }
            let linea = self.obtener_contenido_linea(tamanio)?;
            //esto deberia ir antes o despues del push juani  ?? estaba asi
            lineas.push(linea.clone());
            if linea.contains("NAK")
                || linea.contains("ACK")
                || (linea.contains("done") && !linea.contains("ref")) 
                || linea.contains("ERR")
            {
                break;
            }
        }
        Ok(lineas)
    }
    pub fn responder(&self, lineas: &Vec<String>) -> Result<(), String> {
        if lineas.is_empty() {
            self.flujo
                .lock()
                .unwrap()
                .write_all(String::from("0000").as_bytes()).map_err(|e| e.to_string())?;
            return Ok(());
        }
        for linea in lineas {
            self.flujo.lock().unwrap().write_all(linea.as_bytes()).map_err(|e| e.to_string())?;
        }
        if lineas[0].contains("ref") {
            self.flujo
                .lock()
                .unwrap()
                .write_all(String::from("0000").as_bytes()).map_err(|e| e.to_string())?;
            return Ok(());
        }
        if !lineas[0].contains(&"NAK".to_string())
            && !lineas[0].contains(&"ACK".to_string())
            && !lineas[0].contains(&"done".to_string())
        {
            self.flujo
                .lock()
                .unwrap()
                .write_all(String::from("0000").as_bytes()).map_err(|e| e.to_string())?;
        }
        Ok(())
    }
    
    pub fn enviar_linea(&mut self, linea: &str) -> Result<(), String> {
        self.flujo.lock().unwrap().write_all(linea.as_bytes()).map_err(|e| e.to_string())?;
        Ok(())
    }

    //envia el pack file junto con el flush pkt
    pub fn enviar_pack_file(&self, lineas: Vec<u8>) -> Result<(), String> {
        self.enviar_bytes(&lineas)?;
        if !lineas.starts_with(b"PACK") {
            self.enviar_flush_pkt()?;
        }
        Ok(())
    }

    pub fn obtener_packfile(&self) -> Result<Vec<u8>, String> {
        let mut buffer = Vec::new();
        let mut temp_buffer = [0u8; 1024]; // Tamaño del búfer de lectura

        loop {
            let bytes_read = self
                .flujo
                .lock()
                .map_err(|e| format!("Fallo en el mutex.\n{}\n", e))?
                .read(&mut temp_buffer)
                .map_err(|e| {
                    format!("Fallo en la lectura de la respuesta del servidor.\n{}\n", e)
                })?;

            // Copiar los bytes leídos al búfer principal
            buffer.extend_from_slice(&temp_buffer[0..bytes_read]);

            if buffer.len() > 20 && Packfile::verificar_checksum(&buffer) {
                break;
            }
        }

        Ok(buffer)
    }

    pub fn obtener_obj_ids(&self, lineas: &Vec<String>) -> Vec<String> {
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

        if !capacidades.is_empty() {
            obj_ids[0].push_str(&(" ".to_string() + &capacidades)); // le aniado las capacidades
        }
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

        self.enviar_flush_pkt()?;

        Ok(())
    }

    ///Le añade las capadcidades al primer objeto para cumplir con el protocolo
    fn anadir_capacidades_primer_pedido(&self, pedidos: &mut [String], capacidades: String) {
        pedidos[0].push_str(&(" ".to_string() + &capacidades));
    }
    ///recibi el hash de un commit y le da el formato correcto para hacer el want
    fn dar_formato_de_solicitud(&self, hash_commit: &mut String) -> String {
        io::obtener_linea_con_largo_hex(&("want ".to_string() + hash_commit + "\n"))
    }

    pub fn obtener_haves_pkt(&self, lineas: &Vec<String>) -> Vec<String> {
        let mut haves: Vec<String> = Vec::new();
        for linea in lineas {
            haves.push(io::obtener_linea_con_largo_hex(
                &("have ".to_string() + linea + "\n"),
            ))
        }
        haves
    }

    pub fn enviar_flush_pkt(&self) -> Result<(), String> {
        self.enviar("0000")?;
        Ok(())
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

        self.enviar_flush_pkt()?;

        Ok(())
    }

    ///recibi el hash de un objeto y le da el formato correcto para hacer el have
    fn dar_formato_have(&self, hash_commit: &str) -> String {
        io::obtener_linea_con_largo_hex(&("have ".to_string() + hash_commit + "\n"))
    }
}

#[cfg(test)]
mod test {
    use std::{io::Read, io::Write, path::PathBuf};

    use super::*;

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
        let logger = Arc::new(Logger::new(PathBuf::from("tmp/comunicacion_test02.txt")).unwrap());

        let mut mock = MockTcpStream {
            lectura_data: Vec::new(),
            escritura_data: Vec::new(),
        };

        Comunicacion::new_para_testing(&mut mock, logger)
            .enviar("Hola server, soy siro. Todo bien ??")
            .unwrap();

        assert_eq!(
            "Hola server, soy siro. Todo bien ??".as_bytes(),
            mock.escritura_data.as_slice()
        )
    }

    #[test]

    fn test02_se_obtiene_el_contenido_del_server_de_forma_correcta() {
        let logger = Arc::new(Logger::new(PathBuf::from("tmp/comunicacion_test02.txt")).unwrap());
        let contenido_mock = "000eversion 1 \
        00887217a7c7e582c46cec22a130adf4b9d7d950fba0 HEAD\0multi_ack thin-pack \
        side-band side-band-64k ofs-delta shallow no-progress include-tag \
        00441d3fcd5ced445d1abc402225c0b8a1299641f497 refs/heads/integration \
        003f7217a7c7e582c46cec22a130adf4b9d7d950fba0 refs/heads/master \
        003cb88d2441cac0977faf98efc80305012112238d9d refs/tags/v0.9 \
        003c525128480b96c89e6418b1e40909bf6c5b2d580f refs/tags/v1.0 \
        003fe92df48743b7bc7d26bcaabfddde0a1e20cae47c refs/tags/v1.0^{} \
        0000";

        let mut mock = MockTcpStream {
            lectura_data: contenido_mock.as_bytes().to_vec(),
            escritura_data: Vec::new(),
        };

        let lineas = Comunicacion::new_para_testing(&mut mock, logger)
            .obtener_lineas()
            .unwrap()
            .join("\n");

        let resultado_esperado_de_obtener_lineas = "version 1 \n\
        7217a7c7e582c46cec22a130adf4b9d7d950fba0 HEAD\0multi_ack thin-pack \
       side-band side-band-64k ofs-delta shallow no-progress include-tag \n\
        1d3fcd5ced445d1abc402225c0b8a1299641f497 refs/heads/integration \n\
        7217a7c7e582c46cec22a130adf4b9d7d950fba0 refs/heads/master \n\
        b88d2441cac0977faf98efc80305012112238d9d refs/tags/v0.9 \n\
        525128480b96c89e6418b1e40909bf6c5b2d580f refs/tags/v1.0 \n\
        e92df48743b7bc7d26bcaabfddde0a1e20cae47c refs/tags/v1.0^{} ";

        assert_eq!(resultado_esperado_de_obtener_lineas.to_string(), lineas)
    }

    #[test]
    fn test03_se_envia_correctamente_los_want() {
        let mut mock = MockTcpStream {
            lectura_data: Vec::new(),
            escritura_data: Vec::new(),
        };
        let logger = Arc::new(Logger::new(PathBuf::from("tmp/comunicacion_test02.txt")).unwrap());

        let contenido = vec![
            "74730d410fcb6603ace96f1dc55ea6196122532d".to_string(),
            "7d1665144a3a975c05f1f43902ddaf084e784dbe".to_string(),
            "5a3f6be755bbb7deae50065988cbfa1ffa9ab68a".to_string(),
        ];
        let capacidades = "multi_ack side-band-64k ofs-delta".to_string();

        Comunicacion::new_para_testing(&mut mock, logger)
            .enviar_pedidos_al_servidor_pkt(contenido, capacidades)
            .unwrap();

        let contenido_esperado_enviar_pedido = "\
        0054want 74730d410fcb6603ace96f1dc55ea6196122532d multi_ack side-band-64k ofs-delta\n\
        0032want 7d1665144a3a975c05f1f43902ddaf084e784dbe\n\
        0032want 5a3f6be755bbb7deae50065988cbfa1ffa9ab68a\n\
        0000";

        assert_eq!(
            contenido_esperado_enviar_pedido.as_bytes(),
            mock.escritura_data.as_slice()
        )
    }

    #[test]
    fn test04_se_envia_correctamente_los_have() {
        let mut mock = MockTcpStream {
            lectura_data: Vec::new(),
            escritura_data: Vec::new(),
        };
        let logger = Arc::new(Logger::new(PathBuf::from("tmp/comunicacion_test02.txt")).unwrap());

        let contenido = vec![
            "7e47fe2bd8d01d481f44d7af0531bd93d3b21c01".to_string(),
            "74730d410fcb6603ace96f1dc55ea6196122532d".to_string(),
        ];

        Comunicacion::new_para_testing(&mut mock, logger)
            .enviar_lo_que_tengo_al_servidor_pkt(&contenido)
            .unwrap();

        let contenido_esperado_enviar_lo_que_tengo = "\
        0032have 7e47fe2bd8d01d481f44d7af0531bd93d3b21c01\n\
        0032have 74730d410fcb6603ace96f1dc55ea6196122532d\n\
        0000";

        assert_eq!(
            contenido_esperado_enviar_lo_que_tengo.as_bytes(),
            mock.escritura_data.as_slice()
        )
    }
}

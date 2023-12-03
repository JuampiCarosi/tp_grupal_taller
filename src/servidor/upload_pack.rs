use std::io::{Read, Write};
use crate::tipos_de_dato::comunicacion::Comunicacion;
use crate::tipos_de_dato::packfile;
use crate::utils::io as gir_io;
use crate::utils::strings::eliminar_prefijos;


/// Envia packfile al cliente, 
/// # Argumentos
/// * `dir` - Direccion del repositorio
/// * `comunicacion` - Comunicacion con el cliente
/// # Errores
/// 
pub fn upload_pack<T>(
    dir: String,
    comunicacion: &mut Comunicacion<T>,
) -> Result<(), String>
 where T: Read + Write, {
    let wants = comunicacion.obtener_lineas()?; // obtengo los wants del cliente
    if wants.is_empty() {
        println!("Se termino la conexion");
        return Ok(()); // el cliente esta actualizado
    }
    // ------- CLONE --------
    let lineas_siguientes = comunicacion.obtener_lineas()?;
    if lineas_siguientes[0].clone().contains("done") {
        procesar_pedido_clone(&dir, comunicacion)?;
    }
    else {
        // -------- fetch ----------
        procesar_pedido_fetch(&dir, comunicacion, lineas_siguientes)?;
    }
    println!("Upload pack ejecutado con exito");
    Ok(())
}

fn procesar_pedido_clone<T: Read + Write>(dir: &str, comunicacion: &mut Comunicacion<T>) -> Result<(), String>{ 
    comunicacion.responder(&vec![gir_io::obtener_linea_con_largo_hex("NAK\n")])?; // respondo NAK
    let packfile =
        packfile::Packfile::obtener_pack_entero(&(dir.clone().to_string() + "objects/"))?; // obtengo el packfile
    comunicacion.enviar_pack_file(packfile)?;
    Ok(())
}   

fn procesar_pedido_fetch<T: Read + Write>(dir: &str, comunicacion: &mut Comunicacion<T>, lineas: Vec<String>) -> Result<(), String> {
    let have_objs_ids = eliminar_prefijos(&lineas);
    let respuesta_acks_nak =
    gir_io::obtener_ack(have_objs_ids.clone(), &(dir.to_string() + "objects/"));
    comunicacion.responder(&respuesta_acks_nak)?;
    let _ultimo_done= comunicacion.obtener_lineas()?;
    let faltantes = gir_io::obtener_archivos_faltantes(have_objs_ids, dir);
    let packfile =
        packfile::Packfile::obtener_pack_con_archivos(faltantes, &(dir.to_string() + "objects/"))?;

    comunicacion.enviar_pack_file(packfile)?;
    Ok(())
}

#[cfg(test)]
mod test {
    use std::io::{Read, Write};
    use std::path::PathBuf;
    use std::sync::Arc;
    use crate::tipos_de_dato::{comunicacion::Comunicacion, logger::Logger};
    use super::*;

    struct MockTcpStream {
        lectura_data: Vec<u8>,
    }

    impl Read for MockTcpStream {
        fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
            let bytes_to_read = std::cmp::min(buf.len(), self.lectura_data.len());
            buf[0..bytes_to_read].copy_from_slice(&self.lectura_data[..bytes_to_read]);
            self.lectura_data.drain(..bytes_to_read);
            Ok(bytes_to_read)
        }
    }

    impl Write for MockTcpStream {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            self.lectura_data.extend_from_slice(buf);
            Ok(buf.len())
        }

        fn flush(&mut self) -> std::io::Result<()> {
            self.lectura_data.flush()
        }
    }
    #[test]
    fn test01_clone() {
        let wants = gir_io::obtener_linea_con_largo_hex("4163eb28ec61fd1d0c17cf9b77f4c17e1e338b0\n");
        let test_dir = env!("CARGO_MANIFEST_DIR").to_string() + "/server_test_dir/test03/.gir/"; 

        let mock: MockTcpStream = MockTcpStream {
            lectura_data: Vec::new(),
        };
        let logger = Arc::new(Logger::new(PathBuf::from("tmp/fetch_02.txt")).unwrap());
        let mut comunicacion = Comunicacion::new_para_testing(mock, logger.clone());
        comunicacion.enviar_pedidos_al_servidor_pkt(vec![wants], "".to_string()).unwrap();
        comunicacion.enviar(&gir_io::obtener_linea_con_largo_hex("done\n")).unwrap();
        upload_pack(test_dir, &mut comunicacion).unwrap();
        let respuesta = comunicacion.obtener_lineas().unwrap();
        let respuesta_esperada = vec!["NAK\n".to_string()];
        assert_eq!(respuesta, respuesta_esperada);
        let packfile = comunicacion.obtener_packfile().unwrap();
        assert_eq!(&packfile[..4], "PACK".as_bytes());
    }
   
    #[test]
    fn test02_fetch() {
        let wants = gir_io::obtener_linea_con_largo_hex("4163eb28ec61fd1d0c17cf9b77f4c17e1e338b0");
        let test_dir = env!("CARGO_MANIFEST_DIR").to_string() + "/server_test_dir/test03/.gir/"; 

        let mock: MockTcpStream = MockTcpStream {
            lectura_data: Vec::new(),
        };
        let logger = Arc::new(Logger::new(PathBuf::from("tmp/fetch_02.txt")).unwrap());
        let mut comunicacion = Comunicacion::new_para_testing(mock, logger.clone());
        comunicacion.enviar_pedidos_al_servidor_pkt(vec![wants], "".to_string()).unwrap();
        comunicacion.enviar_lo_que_tengo_al_servidor_pkt(&vec!["8f63722a025d936c53304d40ba3197ffebf194d1\n".to_string()]).unwrap();
        comunicacion.responder(&vec![gir_io::obtener_linea_con_largo_hex("done\n")]).unwrap();
        upload_pack(test_dir, &mut comunicacion).unwrap();
        let respuesta = comunicacion.obtener_lineas().unwrap();
        let respuesta_esperada = vec!["ACK 8f63722a025d936c53304d40ba3197ffebf194d1\n".to_string()];
        assert_eq!(respuesta, respuesta_esperada);
        let packfile = comunicacion.obtener_packfile().unwrap();
        assert_eq!(&packfile[..4], "PACK".as_bytes());
    }

}
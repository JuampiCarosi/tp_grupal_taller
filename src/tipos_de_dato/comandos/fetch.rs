use crate::io::escribir_bytes;
use crate::packfile::Packfile;
use crate::tipos_de_dato::logger::Logger;
use crate::utilidades_path_buf;
use crate::{comunicacion::Comunicacion, io};
use std::io::{Write, Read};
use std::path::PathBuf;
use std::net::TcpStream;
use std::rc::Rc;

pub struct Fetch <T: Write + Read>{
    remoto: String,
    comunicacion: Comunicacion<T>,
    capacidades_local: Vec<String>,
    logger: Rc<Logger>,
}

impl <T:Write + Read>Fetch<T>{
    pub fn new(logger: Rc<Logger>) -> Result<Fetch<TcpStream>, String> {
        let remoto = "origin".to_string();
        //"Por ahora lo hardcoedo necesito el config que no esta en esta rama";

        let direccion_servidor = "127.0.0.1:9418"; // Cambia la dirección IP si es necesario
                                                   //se inicia la comunicacon con servidor
        let comunicacion =
            Comunicacion::<TcpStream>::new_desde_direccion_servidor(direccion_servidor)?;
        //esto ya lo deberia recibir el fetch en realidad

        let capacidades_local = Vec::new();
        //esto lo deberia tener la comunicacion creo yo
        Ok(Fetch {
            remoto,
            comunicacion,
            capacidades_local,
            logger,
        })
    }
    //pòr ahoar para testing, para mi asi deberia ser recibiendo el comunicacion 
    pub fn new_testing(logger: Rc<Logger>, comunicacion: Comunicacion<T>) -> Result<Fetch<T>, String> {
        let remoto = "origin".to_string();
        //"Por ahora lo hardcoedo necesito el config que no esta en esta rama";

        let capacidades_local = Vec::new();
        //esto lo deberia tener la comunicacion creo yo
        Ok(Fetch {
            remoto,
            comunicacion,
            capacidades_local,
            logger,
        })
    }


    // ------------------------------------------------------------- 
    // ------------------------------------------------------------- 
    // ------------------------------------------------------------- 
    // ------------------------------------------------------------- 
    // ------------------------------------------------------------- 
    // ------------------------------------------------------------- 
    // ------------------------------------------------------------- 
    // ------------------------------------------------------------- 
    //verificar si existe /.git
    pub fn ejecutar(&self) -> Result<String, String> {
        self.iniciar_git_upload_pack_con_servidor()?;

        //en caso de clone el commit head se tiene que utilizar
        let (
            capacidades_servidor,
            _commit_head_remoto,
            commits_cabezas_y_dir_rama_asosiado,
            _commits_y_tags_asosiados,
        ) = self.fase_de_descubrimiento()?;
        self.fase_de_negociacion(capacidades_servidor, &commits_cabezas_y_dir_rama_asosiado)?;
        self.recivir_packfile_y_guardar_objetos()?;

        self.actualizar_ramas_locales_del_remoto(&commits_cabezas_y_dir_rama_asosiado)?;

        let mensaje = format!("Fetch ejecutado con exito");
        self.logger.log(mensaje.clone());
        Ok(mensaje)
    }
    // ------------------------------------------------------------- 
    // ------------------------------------------------------------- 
    // ------------------------------------------------------------- 
    // ------------------------------------------------------------- 
    // ------------------------------------------------------------- 
    // ------------------------------------------------------------- 
    // ------------------------------------------------------------- 
    // ------------------------------------------------------------- 



    fn fase_de_negociacion(
        &self,
        capacidades_servidor: Vec<String>,
        commits_cabezas_y_dir_rama_asosiado: &Vec<(String, PathBuf)>,
    ) -> Result<(), String> {
        self.enviar_pedidos(&capacidades_servidor, &commits_cabezas_y_dir_rama_asosiado)?;

        self.enviar_lo_que_tengo()?;

        self.finalizar_pedido()?;
        self.recivir_nack()
    }

    //ACA PARA MI HAY UN PROBLEMA DE RESPONSABILIADADES: COMUNICACION DEBERIA RECIBIR EL PACKETE Y FETCH
    //DEBERIA GUARDAR LAS COSAS, PERO COMO NO ENTIENDO EL CODIGO JAJA DENTRO DE COMUNICACION NO METO MANO
    fn recivir_packfile_y_guardar_objetos(&self) -> Result<(), String> {
        // aca para git daemon hay que poner un recibir linea mas porque envia un ACK repetido (No entiendo por que...)
        println!("Obteniendo paquete..");
        let mut packfile = self.comunicacion.obtener_lineas_como_bytes()?;
        Packfile::new()
            .obtener_paquete_y_escribir(&mut packfile, String::from("./.gir/objects/"))
            .unwrap();

        Ok(())
    }

    ///Envia un mensaje al servidor para avisarle que ya se termino de de mandarle lineas.
    /// Para seguir el protocolo el mensaje que se envia es done
    fn finalizar_pedido(&self) -> Result<(), String> {
        self.comunicacion
            .enviar(&io::obtener_linea_con_largo_hex("done\n"))
    }

    ///Envia todo los objetos (sus hash) que ya se tienen y por lo tanto no es necesario que el servidor manda
    fn enviar_lo_que_tengo(&self) -> Result<(), String> {
        let objetos_directorio =
            io::obtener_objetos_del_directorio("./.gir/objects/".to_string()).unwrap();
        if !objetos_directorio.is_empty() {
            self.comunicacion
                .enviar_lo_que_tengo_al_servidor_pkt(&objetos_directorio)?;
            self.recivir_nack()?;
        } 

        Ok(())
    }

    ///Recibe el la repusta Nack del servidor del envio de HAVE
    fn recivir_nack(&self) -> Result<(), String> {
        //POR AHORA NO HACEMOS, NADA CON ESTO: EVALUAR QUE HACER. SOLO LEERMOS
        //PARA SEGUIR EL FLUJO
        let acks_nak = self.comunicacion.obtener_lineas()?;
        println!("acks_nack: {:?}", acks_nak);
        Ok(())
    }

    ///Envia al servidor todos los commits cabeza de rama que se quieren actulizar junto con las capacidades del
    /// servidor.
    fn enviar_pedidos(
        &self,
        capacidades_servidor: &Vec<String>,
        commits_cabezas_y_dir_rama_asosiado: &Vec<(String, PathBuf)>,
    ) -> Result<(), String> {
        let capacidades_a_usar_en_la_comunicacion =
            self.obtener_capacidades_en_comun_con_el_servidor(capacidades_servidor);
        let commits_de_cabeza_de_rama_faltantes =
            self.obtener_commits_cabeza_de_rama_faltantes(commits_cabezas_y_dir_rama_asosiado)?;

        self.comunicacion.enviar_pedidos_al_servidor_pkt(
            commits_de_cabeza_de_rama_faltantes,
            capacidades_a_usar_en_la_comunicacion,
        )?;
        Ok(())
    }

    ///Obtiene los commits que son necesarios a actulizar y por lo tanto hay que pedirle al servidor esas ramas.
    /// Obtiene aquellos commits que pertenecesen a ramas cuyas cabezas en el servidor apuntan commits distintos
    /// que sus equivalencias en el repositorio local, implicando que la rama local esta desacululizada.
    ///
    /// # Resultado
    ///
    /// - Devuleve un vector con los commits cabezas de las ramas que son necearias actualizar con
    ///     respecto a las del servidor
    fn obtener_commits_cabeza_de_rama_faltantes(
        &self,
        commits_cabezas_y_dir_rama_asosiado: &Vec<(String, PathBuf)>,
    ) -> Result<Vec<String>, String> {
        let mut commits_de_cabeza_de_rama_faltantes: Vec<String> = Vec::new();

        for (commit_cabeza_remoto, dir_rama_asosiada) in commits_cabezas_y_dir_rama_asosiado {
            let dir_rama_asosiada_local =
                self.convertir_de_dir_rama_remota_a_dir_rama_local(dir_rama_asosiada)?;

            if !dir_rama_asosiada_local.exists() {
                commits_de_cabeza_de_rama_faltantes.push(commit_cabeza_remoto.to_string());
                continue;
            }

            let commit_cabeza_local = io::leer_a_string(dir_rama_asosiada_local)?;

            if commit_cabeza_local != commit_cabeza_remoto.to_string() {
                commits_de_cabeza_de_rama_faltantes.push(commit_cabeza_remoto.to_string());
            }
        }

        Ok(commits_de_cabeza_de_rama_faltantes)
    }

    ///compara las capacidades del servidor con las locales y devulve un string con las capacidades en comun
    /// para usar en la comunicacion
    fn obtener_capacidades_en_comun_con_el_servidor(
        &self,
        capacidades_servidor: &Vec<String>,
    ) -> String {
        let mut capacidades_a_usar_en_la_comunicacion: Vec<&str> = Vec::new();

        capacidades_servidor.into_iter().for_each(|capacidad| {
            if self.capacidades_local.contains(&capacidad.to_string()) {
                capacidades_a_usar_en_la_comunicacion.push(capacidad);
            }
        });

        capacidades_a_usar_en_la_comunicacion.join(" ")
    }
    ///Se encarga de la fase de descubrimiento con el servidor, en la cual se recibe del servidor
    /// una lista de referencias.
    /// La primera linea contiene la version del server
    /// La segunda linea recibida tiene el siguiente : 'hash_del_commit_head HEAD'\0'lista de capacida'
    /// Las siguients lineas: 'hash_del_commit_cabeza_de_rama_en_el_servidor'
    ///                        'direccion de la carpeta de la rama en el servidor'
    ///
    /// # Resultado
    /// - vector con las capacidades del servidor
    /// - hash del commit cabeza de rama
    /// - vector de tuplas con los hash del commit cabeza de rama y la direccion de la
    ///     carpeta de la rama en el servidor(ojo!! la direccion para el servidor no para el local)
    /// - vector de tuplas con el hash del commit y el tag asosiado 
    fn fase_de_descubrimiento(
        &self,
    ) -> Result<
        (
            Vec<String>,
            String,
            Vec<(String, PathBuf)>,
            Vec<(String, PathBuf)>,
        ),
        String,
    > {
        let mut lineas_recibidas = self.comunicacion.obtener_lineas()?;
        let version = lineas_recibidas.remove(0); //la version del server
        let segunda_linea = lineas_recibidas.remove(0);

        let (commit_head_remoto, capacidades) =
            self.obtener_commit_head_y_capacidades(&segunda_linea)?;
        
        let (commits_cabezas_y_dir_rama_asosiado, commits_y_tags_asosiados) =
            self.obtener_commits_y_dir_rama_o_tag_asosiados(&lineas_recibidas)?;

        Ok((
            capacidades,
            commit_head_remoto,
            commits_cabezas_y_dir_rama_asosiado,
            commits_y_tags_asosiados,
        ))
    }

    ///Inicia el comando git upload pack con el servidor, mandole al servidor el siguiente mensaje
    /// en formato:
    ///
    /// - ''git-upload-pack 'directorio'\0host='host'\0\0verision='numero de version'\0''
    ///
    fn iniciar_git_upload_pack_con_servidor(&self) -> Result<(), String> {
        let comando = "git-upload-pack";
        let repositorio = "/gir/";
        let host = "example.com";
        let numero_de_version = 1;

        let mensaje = format!(
            "{} {}\0host={}\0\0version={}\0",
            comando, repositorio, host, numero_de_version
        );
        self.comunicacion
            .enviar(&io::obtener_linea_con_largo_hex(&mensaje))?;
        Ok(())
    }

    fn obtener_commits_y_dir_rama_o_tag_asosiados(
        &self,
        lineas_recibidas: &Vec<String>,
    ) -> Result<(Vec<(String, PathBuf)>, Vec<(String, PathBuf)>), String> {
        let mut commits_cabezas_y_dir_rama_asosiados: Vec<(String, PathBuf)> = Vec::new();

        let mut commits_y_tags_asosiados: Vec<(String, PathBuf)> = Vec::new();

        for linea in lineas_recibidas {
            let (commit, dir) = self.obtener_commit_y_dir_asosiado(&linea)?;

            if self.es_la_ruta_a_una_rama(&dir) {
                commits_cabezas_y_dir_rama_asosiados.push((commit, dir));
            } else {
                commits_y_tags_asosiados.push((commit, dir));
            }
        }

        Ok((
            commits_cabezas_y_dir_rama_asosiados,
            commits_y_tags_asosiados,
        ))
    }

    ///Comprueba si dir es el la ruta a una carpeta que corresponde a una rama o a una
    /// tag.
    ///
    /// Si el path contien heads entonces es una rama, devuelve true. Caso contrio es un tag,
    /// devuelve false
    fn es_la_ruta_a_una_rama(&self, dir: &PathBuf) -> bool {
        for componente in dir.iter() {
            if let Some(componente_str) = componente.to_str() {
                if componente_str == "heads" {
                    return true;
                }
            }
        }
        false
    }

    fn convertir_de_dir_rama_remota_a_dir_rama_local(
        &self,
        dir_rama_remota: &PathBuf,
    ) -> Result<PathBuf, String> {
        let carpeta_del_remoto = format!("./.gir/refs/remotes/{}/", self.remoto);
        //"./.gir/refs/remotes/origin/";

        let rama_remota = utilidades_path_buf::obtener_nombre(dir_rama_remota)?;
        let dir_rama_local = PathBuf::from(carpeta_del_remoto + rama_remota.as_str());

        Ok(dir_rama_local)
    }

    fn obtener_commit_head_y_capacidades(
        &self,
        primera_linea: &String,
    ) -> Result<(String, Vec<String>), String> {
        let (commit_head_remoto, capacidades) = primera_linea.split_once('\0').ok_or(format!(
            "Fallo al separar la primera line en commit HEAD y capacidades\n"
        ))?;

        // let commit_head_remoto_sin_head = commit_head_remoto.replace("HEAD", "").trim().to_string();
        
        let capacidades_vector: Vec<String> = capacidades
            .split_whitespace()
            .map(|s| s.to_string())
            .collect();

        Ok((commit_head_remoto.to_string(), capacidades_vector))
    }

    ///Separa el commit del dir asosiado
    ///
    /// # argumento
    ///
    /// referencia: un string con el commit y la rama o tag asosiado. Con el formato:
    ///     "'hash del commit' 'rama_remota/tag'"
    fn obtener_commit_y_dir_asosiado(
        &self,
        referencia: &String,
    ) -> Result<(String, PathBuf), String> {
        let (commit_cabeza_de_rama, dir) = referencia.split_once(' ').ok_or(format!(
            "Fallo al separar el conendio en actualizar referencias\n"
        ))?;

        let dir_path = PathBuf::from(dir.trim());
        Ok((commit_cabeza_de_rama.to_string(), dir_path))
    }

    ///actuliza a donde apuntan las cabeza del rama de las ramas locales pertenecientes al remoto
    fn actualizar_ramas_locales_del_remoto(
        &self,
        commits_cabezas_y_dir_rama_asosiado: &Vec<(String, PathBuf)>,
    ) -> Result<(), String> {
        for (commit_cabeza_de_rama, dir_rama_remota) in commits_cabezas_y_dir_rama_asosiado {
            let dir_rama_local_del_remoto =
                self.convertir_de_dir_rama_remota_a_dir_rama_local(&dir_rama_remota)?;

            io::escribir_bytes(dir_rama_local_del_remoto, commit_cabeza_de_rama)?;
        }

        Ok(())
    }



//     pub fn ejecutar(&mut self) -> Result<String, String> {
//         println!("Se ejecutó el comando fetch");
//         // esto deberia llamar a fetch-pack
//         // let server_address = "127.0.0.1:9418"; // hardcodeado
//         let mut client = TcpStream::connect(("localhost", 9418)).unwrap();
//         let mut comunicacion = Comunicacion::new(client.try_clone().unwrap());

//         // si es un push, tengo que calcular los commits de diferencia entre el cliente y el server, y mandarlos como packfiles.
//         // hay una funcion que hace el calculo
//         // obtener_listas_de_commits
//         // ===============================================================================
//         // EN LUGAR DE GIR HAY QUE PONER EL NOMBRE DE LA CARPETA QUE LO CONTIENE
//         // ===============================================================================
//         let request_data = "git-upload-pack /gir/\0host=example.com\0\0version=1\0"; //en donde dice /.git/ va la dir del repo
//         let request_data_con_largo_hex = io::obtener_linea_con_largo_hex(request_data);

//         client.write_all(request_data_con_largo_hex.as_bytes()).unwrap();
//         let mut refs_recibidas = comunicacion.obtener_lineas().unwrap();

//         if refs_recibidas.len() == 1 {
//             return Ok(String::from("No hay refs"));
//         }
//         println!("refs: {:?}", refs_recibidas);

//         if refs_recibidas.is_empty() {
//             return Err(String::from("No se recibieron referencias"));
//         }
//         let version = refs_recibidas.remove(0);
//         let first_ref = refs_recibidas.remove(0);
//         let referencia_y_capacidades = first_ref.split('\0').collect::<Vec<&str>>();
//         let capacidades = referencia_y_capacidades[1];
//         let diferencias = io::obtener_diferencias_remote(refs_recibidas, "./.gir/".to_string());
//         if diferencias.is_empty(){
//             comunicacion.enviar_flush_pkt().unwrap();
//             return Ok(String::from("El cliente esta actualizado"));
//         }
//         let wants = comunicacion.obtener_wants_pkt(&diferencias, "".to_string()).unwrap();
//         println!("wants: {:?}", wants);
//         comunicacion.responder(wants.clone()).unwrap();

//         let objetos_directorio = io::obtener_objetos_del_directorio("./.gir/objects/".to_string()).unwrap();

//         let haves = comunicacion.obtener_haves_pkt(&objetos_directorio);
//         if !haves.is_empty() {
//             println!("Haves: {:?}", haves);
//             comunicacion.responder(haves).unwrap();
//             let acks_nak = comunicacion.obtener_lineas().unwrap();
//             comunicacion.responder(vec![io::obtener_linea_con_largo_hex("done\n")]).unwrap();
//             // let acks_nak = comunicacion.obtener_lineas().unwrap();
//             println!("acks_nack: {:?}", acks_nak);
//         } else {
//             comunicacion.responder(vec![io::obtener_linea_con_largo_hex("done\n")]).unwrap();
//             let acks_nak = comunicacion.obtener_lineas().unwrap();
//             println!("acks_nack: {:?}", acks_nak);

//         }
        
//         println!("Obteniendo paquete..");
//         let mut packfile = comunicacion.obtener_lineas_como_bytes().unwrap();
//         Packfile::new().obtener_paquete_y_escribir(&mut packfile, String::from("./.gir/objects/")).unwrap();
//         escribir_en_remote_origin_las_referencias(&diferencias);

//         Ok(String::from("Fetch ejecutado con exito"))
//     }
// }

// fn escribir_en_remote_origin_las_referencias(referencias: &Vec<String>) {
//     let remote_origin = "./.gir/refs/remotes/origin/";

//     for referencia in referencias {
//         let referencia_y_contenido = referencia.split_whitespace().collect::<Vec<&str>>();
//         let referencia_con_remote_origin = PathBuf::from(referencia_y_contenido[1]);
//         let nombre_referencia = referencia_con_remote_origin.file_name().unwrap();
//         let dir = PathBuf::from(remote_origin.to_string() + nombre_referencia.to_str().unwrap());
//         println!("Voy a escribir en: {:?}", dir);
//         escribir_bytes(dir, referencia_y_contenido[0]).unwrap();




    
}

#[cfg(test)]

mod test {
    use std::{io::{Read, Write}, path::PathBuf, rc::Rc};

    use crate::{comunicacion::Comunicacion, tipos_de_dato::logger::{self, Logger}};

    use super::Fetch;

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
    fn test01_la_fase_de_descubrimiento_funcion(){
        let contenido_mock = "000eversion 1 \
        00887217a7c7e582c46cec22a130adf4b9d7d950fba0 HEAD\0multi_ack thin-pack \
        side-band side-band-64k ofs-delta shallow no-progress include-tag \
        00441d3fcd5ced445d1abc402225c0b8a1299641f497 refs/heads/integration \
        003f7217a7c7e582c46cec22a130adf4b9d7d950fba0 refs/heads/master \
        003cb88d2441cac0977faf98efc80305012112238d9d refs/tags/v0.9 \
        003c525128480b96c89e6418b1e40909bf6c5b2d580f refs/tags/v1.0 \
        003fe92df48743b7bc7d26bcaabfddde0a1e20cae47c refs/tags/v1.0^{} \
        0000";

        let mock = MockTcpStream {
            lectura_data: contenido_mock.as_bytes().to_vec(),
            escritura_data: Vec::new(),
        };

        let comunicacion = Comunicacion::new(mock);
        let logger = Rc::new(Logger::new(PathBuf::from(".log.txt")).unwrap());
        let (capacidades, commit_head, commits_y_ramas, commits_y_tags) = Fetch::new_testing(logger, comunicacion).unwrap().fase_de_descubrimiento().unwrap();

        let capacidades_esperadas = "multi_ack thin-pack side-band side-band-64k ofs-delta shallow no-progress include-tag";
        assert_eq!(capacidades_esperadas, capacidades.join(" "));
        
        let commit_head_esperado = "7217a7c7e582c46cec22a130adf4b9d7d950fba0";
        assert_eq!(commit_head_esperado, commit_head); 
        
        let commits_y_ramas_esperadas = vec![("1d3fcd5ced445d1abc402225c0b8a1299641f497".to_string(), PathBuf::from("refs/heads/integration")),("7217a7c7e582c46cec22a130adf4b9d7d950fba0".to_string(), PathBuf::from("refs/heads/master"))];
        assert_eq!(commits_y_ramas_esperadas, commits_y_ramas);

        let commits_y_tags_esperados = vec![("b88d2441cac0977faf98efc80305012112238d9d".to_string(), PathBuf::from("refs/tags/v0.9")),("525128480b96c89e6418b1e40909bf6c5b2d580f".to_string(), PathBuf::from("refs/tags/v1.0")),("e92df48743b7bc7d26bcaabfddde0a1e20cae47c".to_string(), PathBuf::from("refs/tags/v1.0^{}".to_string()))];
        assert_eq!(commits_y_tags_esperados, commits_y_tags)
    } 

    #[test]
    fn test02_la_fase_de_negociacion_funciona(){
        
    }
}

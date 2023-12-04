use crate::tipos_de_dato::comando::Ejecutar;
use crate::tipos_de_dato::comunicacion::Comunicacion;
use crate::tipos_de_dato::config::Config;
use crate::tipos_de_dato::logger::Logger;
use crate::tipos_de_dato::packfile::Packfile;
use crate::tipos_de_dato::referencia_commit::ReferenciaCommit;
use crate::utils::{self, io, objects};
use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::PathBuf;
use std::sync::Arc;

const SE_ENVIO_ALGUN_PEDIDO: bool = true;
const NO_SE_ENVIO_NINGUN_PEDIDO: bool = false;
const GIR_FETCH: &str = "gir fetch <remoto>";

pub struct Fetch<T: Write + Read> {
    remoto: String,
    comunicacion: Comunicacion<T>,
    capacidades_local: Vec<String>,
    logger: Arc<Logger>,
}

impl<T: Write + Read> Fetch<T> {
    pub fn new(args: Vec<String>, logger: Arc<Logger>) -> Result<Fetch<TcpStream>, String> {
        Self::verificar_argumentos(&args)?;

        let remoto = Self::obtener_remoto(args)?;
        let url = Self::obtener_url(&remoto)?;

        let capacidades_local = vec!["ofs-delta".to_string()];
        //esto lo deberia tener la comunicacion creo yo

        //fijarse si sigue siendo necesario el arc
        let comunicacion = Comunicacion::<TcpStream>::new_desde_url(&url, logger.clone())?;

        Ok(Fetch {
            remoto,
            comunicacion,
            capacidades_local,
            logger,
        })
    }

    #[cfg(test)]
    fn new_testing(
        mut arg: Vec<String>,
        logger: Arc<Logger>,
        comunicacion: Comunicacion<T>,
    ) -> Result<Fetch<T>, String> {
        let remoto = if !arg.is_empty() {
            arg.remove(0)
        } else {
            "origin".to_string()
        };

        let capacidades_local = Vec::new();

        Ok(Fetch {
            remoto,
            comunicacion,
            capacidades_local,
            logger,
        })
    }

    fn verificar_argumentos(args: &Vec<String>) -> Result<(), String> {
        if args.len() > 1 {
            return Err(format!(
                "Parametros desconocidos {}\n {}",
                args.join(" "),
                GIR_FETCH
            ));
        };
        Ok(())
    }

    ///Le pide al config el url asosiado a la rama
    fn obtener_url(remoto: &str) -> Result<String, String> {
        Config::leer_config()?.obtenet_url_asosiado_remoto(remoto)
    }

    ///obtiene el remoto para el comando, si argumentos lo contiene y es valido lo saca de argumentos. Si no hay argumetos lo saca
    /// del remoto asosiado a la rama actual. Si no esta configura la rama actual para ningun remoto devuleve error.
    fn obtener_remoto(args: Vec<String>) -> Result<String, String> {
        let remoto = if args.len() == 1 {
            Self::verificar_remoto(&args[0])?
        } else {
            Self::obtener_remoto_rama_actual()?
        };
        Ok(remoto)
    }

    ///verifica si el remoto envio por el usario existe
    fn verificar_remoto(remoto: &str) -> Result<String, String> {
        if let false = Config::leer_config()?.existe_remote(remoto) {
            return  Err(format!("Remoto desconocido{}\nSi quiere añadir un nuevo remoto:\n\ngir remote add [<nombre-remote>] [<url-remote>]\n\n", remoto));
        };

        Ok(remoto.to_string())
    }

    ///obtiene el remo asosiado a la rama remota actual. Falla si no existe
    fn obtener_remoto_rama_actual() -> Result<String, String> {
        Config::leer_config()?
            .obtener_remoto_rama_actual()
            .ok_or(format!(
                "La rama actual no se encuentra asosiado a ningun remoto\nUtilice:\n\ngir remote add [<nombre-remote>] [<url-remote>]\n\nDespues:\n\n{}\n\n", GIR_FETCH
            ))
    }

    fn guardar_los_tags(
        &self,
        commits_y_tags_asosiados: &Vec<(String, PathBuf)>,
    ) -> Result<(), String> {
        for (commit, ref_tag) in commits_y_tags_asosiados {
            let dir_tag = PathBuf::from("./.gir/").join(ref_tag);
            utils::io::escribir_bytes(dir_tag, commit)?
        }

        self.logger.log("Escritura de tags en fetch exitosa");
        Ok(())
    }

    fn fase_de_negociacion(
        &self,
        capacidades_servidor: Vec<String>,
        commits_cabezas_y_dir_rama_asosiado: &Vec<(String, PathBuf)>,
        commit_y_tags_asosiado: &Vec<(String, PathBuf)>,
    ) -> Result<bool, String> {
        // no hay pedidos :D
        if !self.enviar_pedidos(
            &capacidades_servidor,
            commits_cabezas_y_dir_rama_asosiado,
            commit_y_tags_asosiado,
        )? {
            return Ok(NO_SE_ENVIO_NINGUN_PEDIDO);
        }

        self.enviar_lo_que_tengo()?;

        self.logger
            .log("Se completo correctamente la fase de negociacion en Fetch");
        Ok(SE_ENVIO_ALGUN_PEDIDO)
    }

    fn recibir_packfile_y_guardar_objetos(&self) -> Result<(), String> {
        // aca para git daemon hay que poner un recibir linea mas porque envia un ACK repetido (No entiendo por que...)
        println!("Obteniendo paquete..");
        let packfile = self.comunicacion.obtener_packfile()?;
        let primeros_bytes = &packfile[..4];
        if primeros_bytes != "PACK".as_bytes() {
            println!(
                "Se recibio: {}",
                String::from_utf8_lossy(packfile.as_slice())
            );
            return Err(format!(
                "Error al recibir el packfile, se recibio: {}",
                String::from_utf8_lossy(packfile.as_slice())
            ));
        }

        self.logger.log("Recepcion del pack file en fetch exitoso");
        Packfile::leer_packfile_y_escribir(&packfile, "./.gir/objects/".to_string()).unwrap();
        Ok(())
    }

    ///Envia un mensaje al servidor para avisarle que ya se termino de de mandarle lineas.
    /// Para seguir el protocolo el mensaje que se envia es done
    fn finalizar_pedido(&self) -> Result<(), String> {
        self.comunicacion
            .enviar(&io::obtener_linea_con_largo_hex("done\n"))
    }

    ///Actuliza el archivo head correspondiente al remoto que se hizo fetch o si no existe lo crea.
    /// Si se hizo fetch del remoto 'san_lorenzo' -> se actuliza o crea el archivo `SAN_LORENZO_HEAD`
    /// con el commit hash cabeza recibido del servidor    
    fn acutualizar_archivo_head_remoto(
        &self,
        commit_head_remoto: &Option<String>,
    ) -> Result<(), String> {
        if let Some(hash) = commit_head_remoto {
            let ubicacion_archivo_head_remoto =
                format!("./.gir/{}_HEAD", self.remoto.to_uppercase());

            println!(
                "ubicacion_archivo_head_remoto: {}",
                ubicacion_archivo_head_remoto
            );
            io::escribir_bytes(ubicacion_archivo_head_remoto, hash)?;
        }

        Ok(())
    }

    ///Envia todo los objetos (sus hash) que ya se tienen y por lo tanto no es necesario que el servidor manda
    fn enviar_lo_que_tengo(&self) -> Result<(), String> {
        //ESTAMOS ENVIANDO TODOS LOS OBJETOS QUE TENEMOS SIN DISTINCION, DE QUE RAMA ESTAN. FUNCIONA
        //PERO SE PODRIA ENVIAR SOLO DE LAS QUE LE PEDISTE
        let objetos = objects::obtener_objetos_del_dir(&PathBuf::from("./.gir/objects"))?;

        if !objetos.is_empty() {
            self.comunicacion
                .enviar_lo_que_tengo_al_servidor_pkt(&objetos)?;
            self.recibir_nack()?;
            self.finalizar_pedido()?
        } else {
            self.finalizar_pedido()?;
            self.recibir_nack()?;
        }
        self.logger.log("Se envio con exito lo que tengo en Fetch");
        Ok(())
    }

    ///Recibe el la repusta Nack del servidor del envio de HAVE
    fn recibir_nack(&self) -> Result<(), String> {
        //POR AHORA NO HACEMOS, NADA CON ESTO: EVALUAR QUE HACER. SOLO LEERMOS
        //PARA SEGUIR EL FLUJO
        let _acks_nak = self.comunicacion.obtener_lineas()?;
        Ok(())
    }

    ///Envia al servidor todos los commits cabeza de rama que se quieren actulizar junto con las capacidades del
    /// servidor.
    /// La operacion devulve un booleando que dice si se mando o no algun pedido. En caso de enviar algun pedido
    /// se devuelve true, en caso de no enviar ninigun pedido(es decir no se quiere nada del server) se devuelve
    /// false
    fn enviar_pedidos(
        &self,
        capacidades_servidor: &[String],
        commits_cabezas_y_dir_rama_asosiado: &Vec<(String, PathBuf)>,
        commit_y_tags_asosiado: &Vec<(String, PathBuf)>,
    ) -> Result<bool, String> {
        let capacidades_a_usar_en_la_comunicacion =
            self.obtener_capacidades_en_comun_con_el_servidor(capacidades_servidor);

        let commits_de_cabeza_de_rama_faltantes =
            self.obtener_commits_cabeza_de_rama_faltantes(commits_cabezas_y_dir_rama_asosiado)?;
        let tags_faltantes = self.obtener_tags_faltantes(commit_y_tags_asosiado)?;

        let pedidos = [
            &commits_de_cabeza_de_rama_faltantes[..],
            &tags_faltantes[..],
        ]
        .concat();

        if pedidos.is_empty() {
            self.comunicacion.enviar_flush_pkt()?;
            self.logger.log(
                "Se completo correctamente el envio de pedidos en Fetch pero no se envio nada",
            );
            return Ok(NO_SE_ENVIO_NINGUN_PEDIDO);
        }

        self.comunicacion
            .enviar_pedidos_al_servidor_pkt(pedidos, capacidades_a_usar_en_la_comunicacion)?;

        self.logger
            .log("Se completo correctamente el envio de pedidos en Fetch");
        Ok(SE_ENVIO_ALGUN_PEDIDO)
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
                utils::ramas::convertir_de_dir_rama_remota_a_dir_rama_local(
                    &self.remoto,
                    dir_rama_asosiada,
                )?;

            if !dir_rama_asosiada_local.exists() {
                commits_de_cabeza_de_rama_faltantes.push(commit_cabeza_remoto.to_string());
                continue;
            }
            let commit_cabeza_local = io::leer_a_string(dir_rama_asosiada_local)?;

            if commit_cabeza_local != *commit_cabeza_remoto {
                commits_de_cabeza_de_rama_faltantes.push(commit_cabeza_remoto.to_string());
            }
        }

        self.logger.log(&format!(
            "Commits ramas faltantes {:?}",
            commits_de_cabeza_de_rama_faltantes
        ));

        Ok(commits_de_cabeza_de_rama_faltantes)
    }

    ///Obtiene los commits que son necesarios a actulizar y por lo tanto hay que pedirle al servidor esas ramas.
    /// Obtiene aquellos commits que pertenecesen a ramas cuyas cabezas en el servidor apuntan commits distintos
    /// que sus equivalencias en el repositorio local, implicando que la rama local esta desacululizada.
    ///
    /// # Resultado
    ///
    /// - Devuleve un vector con los commits cabezas de las ramas que son necearias actualizar con
    ///     respecto a las del servidor
    fn obtener_tags_faltantes(
        &self,
        commit_y_tags_asosiado: &Vec<(String, PathBuf)>,
    ) -> Result<Vec<String>, String> {
        let mut commits_de_tags_faltantes: Vec<String> = Vec::new();

        for (commit_cabeza_remoto, tag_asosiado) in commit_y_tags_asosiado {
            let dir_tag = PathBuf::from("./.gir").join(tag_asosiado);

            if !dir_tag.exists() {
                commits_de_tags_faltantes.push(commit_cabeza_remoto.to_string());
                continue;
            }
            let commit_cabeza_local = io::leer_a_string(dir_tag)?;

            if commit_cabeza_local != *commit_cabeza_remoto {
                commits_de_tags_faltantes.push(commit_cabeza_remoto.to_string());
            }
        }

        self.logger.log(&format!(
            "Commits tags faltantes {:?}",
            commits_de_tags_faltantes
        ));

        Ok(commits_de_tags_faltantes)
    }
    ///compara las capacidades del servidor con las locales y devulve un string con las capacidades en comun
    /// para usar en la comunicacion
    fn obtener_capacidades_en_comun_con_el_servidor(
        &self,
        capacidades_servidor: &[String],
    ) -> String {
        let mut capacidades_a_usar_en_la_comunicacion: Vec<&str> = Vec::new();

        capacidades_servidor.iter().for_each(|capacidad| {
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
            Option<String>,
            ReferenciaCommit,
            ReferenciaCommit,
        ),
        String,
    > {
        let resultado = utils::fase_descubrimiento::fase_de_descubrimiento(&self.comunicacion)?;

        self.logger.log(&format!(
            "Se ejecuto correctamte la fase de decubrimiento en Fech: {:?}",
            resultado
        ));

        Ok(resultado)
    }

    ///actuliza a donde apuntan las cabeza del rama de las ramas locales pertenecientes al remoto
    fn actualizar_ramas_locales_del_remoto(
        &self,
        commits_cabezas_y_dir_rama_asosiado: &Vec<(String, PathBuf)>,
    ) -> Result<(), String> {
        for (commit_cabeza_de_rama, dir_rama_remota) in commits_cabezas_y_dir_rama_asosiado {
            let dir_rama_local_del_remoto =
                utils::ramas::convertir_de_dir_rama_remota_a_dir_rama_local(
                    &self.remoto,
                    dir_rama_remota,
                )?;

            io::escribir_bytes(dir_rama_local_del_remoto, commit_cabeza_de_rama)?;
        }

        self.logger
            .log("Actualizacion de ramas remotas en fetch exitosa");
        Ok(())
    }
}

impl Ejecutar for Fetch<TcpStream> {
    fn ejecutar(&mut self) -> Result<String, String> {
        self.logger.log("Se ejecuto el comando fetch");
        self.comunicacion.iniciar_git_upload_pack_con_servidor()?;

        let (
            capacidades_servidor,
            commit_head_remoto,
            commits_cabezas_y_dir_rama_asosiado,
            commits_y_tags_asosiados,
        ) = self.fase_de_descubrimiento()?;

        if !self.fase_de_negociacion(
            capacidades_servidor,
            &commits_cabezas_y_dir_rama_asosiado,
            &commits_y_tags_asosiados,
        )? {
            return Ok(String::from("El cliente esta actualizado"));
        }

        self.recibir_packfile_y_guardar_objetos()?;

        self.actualizar_ramas_locales_del_remoto(&commits_cabezas_y_dir_rama_asosiado)?;

        self.guardar_los_tags(&commits_y_tags_asosiados)?;

        self.acutualizar_archivo_head_remoto(&commit_head_remoto)?;

        let mensaje = "Fetch ejecutado con exito".to_string();
        self.logger.log(&mensaje);
        Ok(mensaje)
    }
}

#[cfg(test)]

mod test {
    use std::{path::PathBuf, sync::Arc};

    use crate::{
        tipos_de_dato::{comunicacion::Comunicacion, logger::Logger},
        utils::{self, testing::MockTcpStream},
    };

    use super::Fetch;

    #[test]
    fn test_03_los_tags_se_gurdan_correctamtene() {
        let logger = Arc::new(Logger::new(PathBuf::from("tmp/fetch_03.txt")).unwrap());
        utils::testing::limpiar_archivo_gir(logger.clone());

        let tag_1 = "v0.9".to_string();
        let tag_1_contenido = "b88d2441cac0977faf98efc80305012112238d9d".to_string();
        let tag_2 = "v1.0".to_string();
        let tag_2_contenido = "525128480b96c89e6418b1e40909bf6c5b2d580f".to_string();

        let commits_y_tags = vec![
            (
                tag_1_contenido.clone(),
                PathBuf::from(format!("refs/tags/{}", tag_1)),
            ),
            (
                tag_2_contenido.clone(),
                PathBuf::from(format!("refs/tags/{}", tag_2)),
            ),
        ];

        let mock = MockTcpStream {
            lectura_data: Vec::new(),
            escritura_data: Vec::new(),
        };

        let comunicacion = Comunicacion::new_para_testing(mock, logger.clone());
        Fetch::new_testing(vec![], logger, comunicacion)
            .unwrap()
            .guardar_los_tags(&commits_y_tags)
            .unwrap();

        assert!(utils::tags::existe_tag(&tag_1));
        let tag_1_contenido_obtenido =
            utils::io::leer_a_string(format!("./.gir/refs/tags/{}", tag_1)).unwrap();
        assert_eq!(tag_1_contenido_obtenido, tag_1_contenido);

        assert!(utils::tags::existe_tag(&tag_2));
        let tag_2_contenido_obtenido =
            utils::io::leer_a_string(format!("./.gir/refs/tags/{}", tag_2)).unwrap();
        assert_eq!(tag_2_contenido_obtenido, tag_2_contenido);
    }

    #[test]
    fn test_04_los_ramas_remotas_se_escriben_correctamente() {
        let logger = Arc::new(Logger::new(PathBuf::from("tmp/fetch_04.txt")).unwrap());
        utils::testing::limpiar_archivo_gir(logger.clone());

        let remoto = "san-siro".to_string();
        let rama_remota = "tomate".to_string();
        let rama_contenido = "b88d2441cac0977faf98efc80305012112238d9d".to_string();

        let commits_y_ramas = vec![(
            rama_contenido.clone(),
            PathBuf::from(format!("refs/heads/{}", rama_remota)),
        )];

        let mock = MockTcpStream {
            lectura_data: Vec::new(),
            escritura_data: Vec::new(),
        };

        let comunicacion = Comunicacion::new_para_testing(mock, logger.clone());
        Fetch::new_testing(vec![remoto.clone()], logger, comunicacion)
            .unwrap()
            .actualizar_ramas_locales_del_remoto(&commits_y_ramas)
            .unwrap();

        let rama_contendio_obtenido =
            utils::io::leer_a_string(format!("./.gir/refs/remotes/{}/{}", remoto, rama_remota))
                .unwrap();
        assert_eq!(rama_contendio_obtenido, rama_contenido);
    }

    #[test]
    fn test_05_los_ramas_remotas_se_actualizan_correctamente() {
        let logger = Arc::new(Logger::new(PathBuf::from("tmp/fetch_05.txt")).unwrap());
        utils::testing::limpiar_archivo_gir(logger.clone());

        let remoto = "san-siro".to_string();
        let rama_remota = "tomate".to_string();
        let rama_contenido_actualizar = "b88d2441cac0977faf98efc80305012112238d9d".to_string();
        utils::testing::escribir_rama_remota(&remoto, &rama_remota);

        let commits_y_ramas = vec![(
            rama_contenido_actualizar.clone(),
            PathBuf::from(format!("refs/heads/{}", rama_remota)),
        )];

        let mock = MockTcpStream {
            lectura_data: Vec::new(),
            escritura_data: Vec::new(),
        };

        let comunicacion = Comunicacion::new_para_testing(mock, logger.clone());
        Fetch::new_testing(vec![remoto.clone()], logger, comunicacion)
            .unwrap()
            .actualizar_ramas_locales_del_remoto(&commits_y_ramas)
            .unwrap();

        let rama_contendio_obtenido =
            utils::io::leer_a_string(format!("./.gir/refs/remotes/{}/{}", remoto, rama_remota))
                .unwrap();
        assert_eq!(rama_contendio_obtenido, rama_contenido_actualizar);
    }

    #[test]
    fn test_05_los_ramas_remotas_se_escriben_correctamente() {
        let logger = Arc::new(Logger::new(PathBuf::from("tmp/fetch_05.txt")).unwrap());
        utils::testing::limpiar_archivo_gir(logger.clone());

        let remoto = "san-siro".to_string();
        let rama_remota = "tomate".to_string();
        let rama_contenido = "b88d2441cac0977faf98efc80305012112238d9d".to_string();

        let commits_y_ramas = vec![(
            rama_contenido.clone(),
            PathBuf::from(format!("refs/heads/{}", rama_remota)),
        )];

        let mock = MockTcpStream {
            lectura_data: Vec::new(),
            escritura_data: Vec::new(),
        };

        let comunicacion = Comunicacion::new_para_testing(mock, logger.clone());
        Fetch::new_testing(vec![remoto.clone()], logger, comunicacion)
            .unwrap()
            .actualizar_ramas_locales_del_remoto(&commits_y_ramas)
            .unwrap();

        let rama_contendio_obtenido =
            utils::io::leer_a_string(format!("./.gir/refs/remotes/{}/{}", remoto, rama_remota))
                .unwrap();
        assert_eq!(rama_contendio_obtenido, rama_contenido);
    }
    // #[test]
    // fn test03_la_fase_de_negociacion_funciona(){
    //     let nuevo_dir = "test03_fetch";
    //     let viejo_dir = crear_y_cambiar_directorio(nuevo_dir);

    //     let mock = MockTcpStream {
    //         lectura_data: Vec::new(),
    //         escritura_data: Vec::new(),
    //     };

    //     let comunicacion = Comunicacion::new_para_testing(mock);
    //     let logger = Rc::new(Logger::new(PathBuf::from(".log.txt")).unwrap());
    //     let capacidades_servidor = vec!["multi_ack".to_string(), "thin-pack".to_string(), "side-band".to_string(), "side-band-64k".to_string(), "ofs-delta".to_string(), "shallow".to_string(), "no-progress".to_string(),  "include-tag".to_string()];
    //     let commits_y_ramas = vec![("1d3fcd5ced445d1abc402225c0b8a1299641f497".to_string(), PathBuf::from("refs/heads/integration")),("7217a7c7e582c46cec22a130adf4b9d7d950fba0".to_string(), PathBuf::from("refs/heads/master"))];

    //     Fetch::new_testing(logger, comunicacion).unwrap().fase_de_negociacion(capacidades_servidor, &commits_y_ramas).unwrap();

    //     volver_al_viejo_dir_y_borrar_el_nuevo(nuevo_dir, viejo_dir);
    // }

    // fn volver_al_viejo_dir_y_borrar_el_nuevo(nuevo_dir: &str, viejo_dir: PathBuf) {
    //     std::env::set_current_dir(viejo_dir).unwrap();
    //     std::fs::remove_dir_all(nuevo_dir).unwrap();
    // }

    // fn crear_y_cambiar_directorio(nombre: &str)-> PathBuf{
    //     let viejo_dir = env::current_dir().unwrap();

    //     fs::create_dir_all(nombre).unwrap();
    //     std::env::set_current_dir(nombre).unwrap();

    //     viejo_dir
    // }
}

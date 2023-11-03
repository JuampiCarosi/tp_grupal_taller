use crate::io::escribir_bytes;
use crate::tipos_de_dato::logger::Logger;
use crate::utilidades_path_buf;
use crate::{
    comunicacion::Comunicacion, io, packfile, tipos_de_dato::comandos::write_tree,
    tipos_de_dato::objetos::tree::Tree,
};
use std::io::Write;
use std::net::TcpStream;
use std::path::PathBuf;
use std::rc::Rc;

use super::{commit, remote};

pub struct Fetch {
    remoto: String,
    comunicacion: Comunicacion,
    capacidades_local: Vec<String>,
    logger: Rc<Logger>,
}

impl Fetch {
    pub fn new(logger: Rc<Logger>) -> Result<Fetch, String> {
        let remoto = "origin".to_string();
        //"Por ahora lo hardcoedo necesito el config que no esta en esta rama";

        let direccion_servidor = "127.0.0.1:9418"; // Cambia la dirección IP si es necesario
                                                   //se inicia la comunicacon con servidor
        let mut comunicacion = Comunicacion::new_desde_direccion_servidor(direccion_servidor)?;
                                                    //esto ya lo deberia recibir el fetch en realidad 

        let capacidades_local = Vec::new();
            //esto lo deberia tener la comunicacion creo yo 
        Ok(Fetch { remoto , comunicacion, capacidades_local, logger})
    }

    //verificar si existe /.git
    pub fn ejecutar(&mut self) -> Result<String, String> {
        
        self.iniciar_git_upload_pack_con_servidor()?;

        //en caso de clone el commit head se tiene que utilizar
        let (capacidades_servidor, commit_head_remoto, commits_cabezas_y_dir_rama_asosiado) =
            self.fase_de_descubrimiento()?;

        self.fase_de_negociacion(capacidades_servidor, commits_cabezas_y_dir_rama_asosiado)?;
        
        self.recivir_packfile_y_guardar_objetos()?;

        self.actualizar_ramas_locales_del_remoto(&commits_cabezas_y_dir_rama_asosiado)?;
        
        let mensaje = format!("Fetch ejecutado con exito");
        self.logger.log(mensaje);
        Ok(mensaje)
    }

    fn fase_de_negociacion(&mut self, capacidades_servidor: Vec<&str>, commits_cabezas_y_dir_rama_asosiado: Vec<(&str, PathBuf)>) -> Result<(), String> {
        self.enviar_pedidos(&capacidades_servidor, &commits_cabezas_y_dir_rama_asosiado)?;

        self.enviar_lo_que_tengo()?;

        self.finalizar_pedido()

    }

    //ACA PARA MI HAY UN PROBLEMA DE RESPONSABILIADADES: COMUNICACION DEBERIA RECIBIR EL PACKETE Y FETCH 
    //DEBERIA GUARDAR LAS COSAS, PERO COMO NO ENTIENDO EL CODIGO JAJA DENTRO DE COMUNICACION NO METO MANO 
    fn recivir_packfile_y_guardar_objetos(&mut self) -> Result<(), String> {
        // aca para git daemon hay que poner un recibir linea mas porque envia un ACK repetido (No entiendo por que...)
        println!("Obteniendo paquete..");
        let mut packfile = self.comunicacion.obtener_lineas_como_bytes()?;
        self.comunicacion
            .obtener_paquete_y_escribir(&mut packfile, String::from("./.gir/objects/"))
            .unwrap();

        Ok(())
    }
    
    ///Envia un mensaje al servidor para avisarle que ya se termino de de mandarle lineas. 
    /// Para seguir el protocolo el mensaje que se envia es done 
    fn finalizar_pedido(&self)->Result<(),String>{
        self.comunicacion.enviar(&io::obtener_linea_con_largo_hex("done\n"))
    }
    
    ///Envia todo los objetos (sus hash) que ya se tienen y por lo tanto no es necesario que el servidor manda
    fn enviar_lo_que_tengo(&self) -> Result<(), String>{
        let objetos_directorio =
            io::obtener_objetos_del_directorio("./.gir/objects/".to_string()).unwrap();

        if !objetos_directorio.is_empty(){
            self.comunicacion.enviar_lo_que_tengo_al_servidor_pkt(&objetos_directorio)?;
            self.recivir_nack();
        }

        Ok(())
    }

    ///Recibe el la repusta Nack del servidor del envio de HAVE
    fn recivir_nack(&self)->Result<(), String>{
        //POR AHORA NO HACEMOS, NADA CON ESTO: EVALUAR QUE HACER. SOLO LEERMOS 
        //PARA SEGUIR EL FLUJO
        let acks_nak = self.comunicacion.obtener_lineas()?;
        println!("acks_nack: {:?}", acks_nak);
        Ok(())
    }
    ///Envia al servidor todos los commits cabeza de rama que se quieren actulizar junto con las capacidades del 
    /// servidor. 
    fn enviar_pedidos(&self, capacidades_servidor: &Vec<&str>, commits_cabezas_y_dir_rama_asosiado: &Vec<(&str,PathBuf)>)->Result<(), String>{
        let capacidades_a_usar_en_la_comunicacion = self.obtener_capacidades_en_comun_con_el_servidor(capacidades_servidor);
        let commits_de_cabeza_de_rama_faltantes = self.obtener_commits_cabeza_de_rama_faltantes(commits_cabezas_y_dir_rama_asosiado)?;

        self.comunicacion.enviar_pedidos_al_servidor_pkt(&commits_de_cabeza_de_rama_faltantes, capacidades_a_usar_en_la_comunicacion)?;
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
    fn obtener_commits_cabeza_de_rama_faltantes(&self, commits_cabezas_y_dir_rama_asosiado: &Vec<(&str, PathBuf)>)->Result<Vec<String>,String>{
        let mut commits_de_cabeza_de_rama_faltantes:Vec<String> = Vec::new();
        
        for (commit_cabeza_remoto, dir_rama_asosiada) in commits_cabezas_y_dir_rama_asosiado{
            let dir_rama_asosiada_local = self.convertir_de_dir_rama_remota_a_dir_rama_local(dir_rama_asosiada)?;
            
            if !dir_rama_asosiada_local.exists(){
                commits_de_cabeza_de_rama_faltantes.push(commit_cabeza_remoto.to_string());
                
            }

            let commit_cabeza_local = io::leer_a_string(dir_rama_asosiada_local)?;

            if commit_cabeza_local != commit_cabeza_remoto.to_string(){
                commits_de_cabeza_de_rama_faltantes.push(commit_cabeza_remoto.to_string());
            }
        };

        Ok(commits_de_cabeza_de_rama_faltantes)
    }

    ///compara las capacidades del servidor con las locales y devulve un string con las capacidades en comun 
    /// para usar en la comunicacion
    fn obtener_capacidades_en_comun_con_el_servidor(&self, capacidades_servidor: &Vec<&str>)->String{
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
    /// La primera linea recibida tiene el siguiente : 'hash_del_commit_head HEAD'\0'lista de capacida'
    /// Las siguients lineas: 'hash_del_commit_cabeza_de_rama_en_el_servidor'
    ///                        'direccion de la carpeta de la rama en el servidor'
    ///
    /// # Resultado
    /// - vector con las capacidades del servidor
    /// - hash del commit cabeza de rama
    /// -vector de tuplas con los hash del commit cabeza de rama y la direccion de la
    ///     carpeta de la rama en el servidor(ojo!! la direccion para el servidor no para el local)
    fn fase_de_descubrimiento(
        &mut self,
    ) -> Result<(Vec<&str>, &str, Vec<(&str, PathBuf)>), String> {
        let mut lineas_recibidas = self.comunicacion.obtener_lineas()?;

        let primera_linea = lineas_recibidas.remove(0);

        let (commit_head_remoto, capacidades) =
            self.obtener_commit_head_y_capacidades(&primera_linea)?;

        let commits_cabezas_y_dir_rama_asosiado =
            self.obtener_commits_cabezas_y_dirs_ramas_asosiadas(&lineas_recibidas)?;

        Ok((
            capacidades,
            commit_head_remoto,
            commits_cabezas_y_dir_rama_asosiado,
        ))
    }
    
    ///Inicia el comando git upload pack con el servidor, mandole al servidor el siguiente mensaje
    /// en formato:
    ///
    /// - ''git-upload-pack 'directorio'\0host='host'\0\0verision='numero de version'\0''
    ///
    fn iniciar_git_upload_pack_con_servidor(
        &self
    ) -> Result<(), String> {
        let comando = "git-upload-pack";
        let repositorio = "./gir";
        let host = "example.com";
        let numero_de_version = 1;

        let mensaje = format!(
            "{} {}\0host={}\0\0verision={}\0",
            comando, repositorio, host, numero_de_version
        );
        self.comunicacion.enviar(&io::obtener_linea_con_largo_hex(&mensaje))?;
        Ok(())
    }

    fn obtener_commits_cabezas_y_dirs_ramas_asosiadas(
        &mut self,
        lineas_recibidas: &Vec<String>,
    ) -> Result<Vec<(&str, PathBuf)>, String> {
        let mut commits_cabezas_y_dir_rama_asosiado: Vec<(&str, PathBuf)> = Vec::new();
        for linea in *lineas_recibidas {
            let (commit_cabeza, dir_rama) =
                self.obtener_commit_cabeza_y_dir_rama_asosiado(&linea)?;
            commits_cabezas_y_dir_rama_asosiado.push((commit_cabeza, dir_rama));
        }

        Ok(commits_cabezas_y_dir_rama_asosiado)
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
    ) -> Result<(&str, Vec<&str>), String> {
        let (commit_head_remoto, capacidades) = primera_linea.split_once('\0').ok_or(format!(
            "Fallo al separar la primera line en commit HEAD y capacidades\n"
        ))?;

        let capacidades_vector = capacidades.split_whitespace().collect();
        Ok((commit_head_remoto, capacidades_vector))
    }

    fn obtener_commit_cabeza_y_dir_rama_asosiado(
        &self,
        referencia: &String,
    ) -> Result<(&str, PathBuf), String> {
        let (commit_cabeza_de_rama, rama_remota) = referencia.split_once(' ').ok_or(
            format!("Fallo al separar el conendio en actualizar referencias\n"),
        )?;

        let rama_remota_path = PathBuf::from(rama_remota);
        Ok((commit_cabeza_de_rama, rama_remota_path))
    }

    ///actuliza a donde apuntan las cabeza del rama de las ramas locales pertenecientes al remoto
    fn actualizar_ramas_locales_del_remoto(
        &self,
        commits_cabezas_y_dir_rama_asosiado: &Vec<(&str, PathBuf)>,
    ) -> Result<(), String> {
        for (commit_cabeza_de_rama, dir_rama_remota) in commits_cabezas_y_dir_rama_asosiado {
            let dir_rama_local_del_remoto =
                self.convertir_de_dir_rama_remota_a_dir_rama_local(&dir_rama_remota)?;

            io::escribir_bytes(dir_rama_local_del_remoto, commit_cabeza_de_rama)?;
        }

        Ok(())
    }
}

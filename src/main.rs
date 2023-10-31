use gir::{comunicacion::Comunicacion, io, packfile, tipos_de_dato::objetos::tree::Tree, tipos_de_dato::comandos::write_tree};
use std::io::Write;
use std::path::PathBuf;
use std::rc::Rc;
use std::{char::decode_utf16, net::TcpStream};
use gir::tipos_de_dato::{comando::Comando, logger::Logger};


//extrae la ubiacion del archivo log seteada en el archivo config. En caso de error
// devuelve una direccion default = .log
fn obtener_dir_archivo_log(ubicacion_config: PathBuf) -> String {
    let mut dir_archivo_log = ".log".to_string();

    let contenido_config = match io::leer_a_string(ubicacion_config) {
        Ok(contenido_config) => contenido_config,
        Err(_) => return dir_archivo_log,
    };

    for linea_config in contenido_config.lines() {
        if linea_config.trim().starts_with("log") {
            if let Some(dir_archivo_log_config) = linea_config.split('=').nth(1) {
                dir_archivo_log = dir_archivo_log_config.trim().to_string();
                break;
            }
        }
    }

    dir_archivo_log
}

fn main() -> Result<(), String> {
    let args = std::env::args().collect::<Vec<String>>();
    let logger = Rc::new(Logger::new(PathBuf::from("log.txt"))?);

    let mut comando = match Comando::new(args, logger.clone()) {
        Ok(comando) => comando,
        Err(err) => {
            logger.log(err);
            return Ok(());
        }
    };

    match comando.ejecutar() {
        Ok(mensaje) => {
            println!("{}", mensaje);
            logger.log(mensaje);
        }
        Err(mensaje) => {
            println!("ERROR: {}", mensaje);
            logger.log(mensaje);
        }
    };

    Ok(())
}
// fn main() -> std::io::Result<()> {
//     // Configura la dirección del servidor y el puerto
//     let server_address = "127.0.0.1:9418"; // Cambia la dirección IP si es necesario

//     // Conéctate al servidor
//     let mut client = TcpStream::connect(server_address)?;
//     let mut comunicacion = Comunicacion::new(client.try_clone().unwrap());

//     // si es un push, tengo que calcular los commits de diferencia entre el cliente y el server, y mandarlos como packfiles.
//     // hay una funcion que hace el calculo 
//     // obtener_listas_de_commits

//     let request_data = "git-upload-pack /.gir/\0host=example.com\0\0version=1\0"; //en donde dice /.git/ va la dir del repo
//     let request_data_con_largo_hex = io::obtener_linea_con_largo_hex(request_data);

//     client.write_all(request_data_con_largo_hex.as_bytes())?;
//     let refs_recibidas = comunicacion.obtener_lineas().unwrap();

//     let capacidades = refs_recibidas[0].split("\0").collect::<Vec<&str>>()[1];
//     let wants = comunicacion.obtener_wants_pkt(&refs_recibidas, capacidades.to_string()).unwrap();
//     comunicacion.responder(wants.clone()).unwrap();
//     // let objetos_directorio = io::obtener_objetos_del_directorio("./.git/objects/".to_string()).unwrap();
//     // let haves = comunicacion.obtener_haves_pkt(&objetos_directorio);    
    
//     // comunicacion.responder(haves).unwrap();
//     // if clone { 
//     //     envio done previo obtencion de nak
//     // } else { 
//     //     obtengo acks/nak y envio done
//     // }
//     // ESTO ES EN EL CASO EN EL QUE SEA UN CLONE
//     comunicacion.responder(vec![io::obtener_linea_con_largo_hex("done")]).unwrap();
//     let acks_nak = comunicacion.obtener_lineas().unwrap();
//     println!("acks_nack: {:?}", acks_nak);

//     println!("Obteniendo paquete..");
//     let mut packfile = comunicacion.obtener_lineas_como_bytes().unwrap();
//     comunicacion.obtener_paquete(&mut packfile).unwrap();

//     let head_dir: String = String::from("/home/juani/git/HEAD");

//     let ref_head: String = if refs_recibidas[0].contains("HEAD") {
//         refs_recibidas[0].split("\0").collect::<Vec<&str>>()[0].to_string().split(" ").collect::<Vec<&str>>()[0].to_string()
//     } else {
//         refs_recibidas[0].split("\0").collect::<Vec<&str>>()[0].to_string()
//     };
//     io::escribir_bytes(PathBuf::from(head_dir), ref_head.clone().as_bytes()).unwrap();
//     println!("ref_head: {:?}", ref_head);
 
//     let tree_hash = write_tree::conseguir_arbol_padre_from_ult_commit(ref_head.clone());
//     println!("tree_hash: {:?}", tree_hash);
//     let tree: Tree = Tree::from_hash(tree_hash, PathBuf::from("/home/juani/git/objects")).unwrap();
//     match tree.escribir_en_directorio() {
//         Ok(_) => {},
//         Err(e) => {println!("Error al escribir el arbol: {}", e);}
//     }



//     Ok(())
// }



// fn obtener_listas_de_commits(branch: &String) -> Result<Vec<String>, String> {
//     let ruta = format!(".gir/refs/heads/{}", branch);
//     let mut ultimo_commit = leer_a_string(path::Path::new(&ruta))?;

//     if ultimo_commit.is_empty() {
//         return Ok(Vec::new());
//     }
//     let mut historial_commits: Vec<String> = Vec::new();
//     loop {
//         let contenido = utilidades_de_compresion::descomprimir_objeto(ultimo_commit.clone())?;
//         let siguiente_padre = Log::conseguir_padre_desde_contenido_commit(&contenido);
//         historial_commits.push(ultimo_commit.clone());
//         if siguiente_padre.is_empty() {
//             break;
//         }
//         ultimo_commit = siguiente_padre.to_string();
//     }
//     Ok(historial_commits)
// }
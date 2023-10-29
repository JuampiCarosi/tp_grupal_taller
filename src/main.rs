use gir::{comunicacion::Comunicacion, io, packfile};
use std::io::Write;
use std::path::PathBuf;
use std::{char::decode_utf16, net::TcpStream};
fn main() -> std::io::Result<()> {
    // Configura la dirección del servidor y el puerto
    let server_address = "127.0.0.1:9418"; // Cambia la dirección IP si es necesario

    // Conéctate al servidor
    let mut client = TcpStream::connect(server_address)?;
    let mut comunicacion = Comunicacion::new(client.try_clone().unwrap());
    // Envía datos al servidor (reemplaza esto por tus datos Git)
    let request_data = "git-upload-pack /.git/\0host=example.com\0\0version=1\0";
    let request_data_con_largo_hex = io::obtener_linea_con_largo_hex(request_data);

    client.write_all(request_data_con_largo_hex.as_bytes())?;
    let refs_recibidas = comunicacion.obtener_lineas().unwrap();
    println!("refs_recibidas: {:?}", refs_recibidas);

    // let head =  refs_recibidas[0].split('\0').split_whitespace(); 
    if refs_recibidas[0].contains("HEAD") {
        println!("Hola");
        let ref_head = refs_recibidas[0].split("\0").collect::<Vec<&str>>()[0].to_string().split(" ").collect::<Vec<&str>>()[0].to_string();
        let dir: String = String::from("/home/juani/gir/HEAD");
        io::escribir_bytes(PathBuf::from(dir), ref_head.as_bytes()).unwrap();
    }
    // io::crear_archivo(PathBuf::from("home/juani/.git/".to_string().push_str(ref_head.clone()))).unwrap();


    let capacidades = refs_recibidas[0].split("\0").collect::<Vec<&str>>()[1];
    let wants = comunicacion.obtener_wants_pkt(&refs_recibidas, capacidades.to_string()).unwrap();
    comunicacion.responder(wants.clone()).unwrap();
    // let objetos_directorio = io::obtener_objetos_del_directorio("./.git/objects/".to_string()).unwrap();
    // let haves = comunicacion.obtener_haves_pkt(&objetos_directorio);    
    // comunicacion.responder(haves).unwrap();

    // ESTO ES EN EL CASO EN EL QUE SEA UN CLONE
    comunicacion.responder(vec![io::obtener_linea_con_largo_hex("done")]).unwrap();
    let nak = comunicacion.obtener_lineas().unwrap();
    println!("nak: {:?}", nak);

    println!("Obteniendo paquete..");
    let mut packfile = comunicacion.obtener_lineas_como_bytes().unwrap();
    comunicacion.obtener_paquete(&mut packfile).unwrap();

    Ok(())
}

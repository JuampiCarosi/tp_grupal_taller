use gir::{comunicacion::Comunicacion, io, packfile, tipos_de_dato::objetos::tree::Tree, tipos_de_dato::comandos::write_tree};
use std::io::Write;
use std::path::PathBuf;
use std::{char::decode_utf16, net::TcpStream};

fn main() -> std::io::Result<()> {
    // Configura la dirección del servidor y el puerto
    let server_address = "127.0.0.1:9418"; // Cambia la dirección IP si es necesario

    // Conéctate al servidor
    let mut client = TcpStream::connect(server_address)?;
    let mut comunicacion = Comunicacion::new(client.try_clone().unwrap());

    let request_data = "git-upload-pack /.git/\0host=example.com\0\0version=1\0"; //en donde dice /.git/ va la dir del repo
    let request_data_con_largo_hex = io::obtener_linea_con_largo_hex(request_data);

    client.write_all(request_data_con_largo_hex.as_bytes())?;
    let refs_recibidas = comunicacion.obtener_lineas().unwrap();

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

    let head_dir: String = String::from("/home/juani/git/HEAD");

    let ref_head: String = if refs_recibidas[0].contains("HEAD") {
        refs_recibidas[0].split("\0").collect::<Vec<&str>>()[0].to_string().split(" ").collect::<Vec<&str>>()[0].to_string()
    } else {
        refs_recibidas[0].split("\0").collect::<Vec<&str>>()[0].to_string()
    };
    io::escribir_bytes(PathBuf::from(head_dir), ref_head.clone().as_bytes()).unwrap();
    println!("ref_head: {:?}", ref_head);
 
    let tree_hash = write_tree::conseguir_arbol_padre_from_ult_commit(ref_head.clone());
    println!("tree_hash: {:?}", tree_hash);
    let tree: Tree = Tree::from_hash(tree_hash, PathBuf::from("/home/juani/git/objects")).unwrap();
    match tree.escribir_en_directorio() {
        Ok(_) => {},
        Err(e) => {println!("Error al escribir el arbol: {}", e);}
    }
    Ok(())
}

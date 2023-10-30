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
    let tree: Tree = Tree::from_hash(tree_hash, PathBuf::from("/home/juani/git")).unwrap();
    // match tree.escribir_en_directorio() {
    //     Ok(_) => {},
    //     Err(e) => {println!("Error al escribir el arbol: {}", e);}
    // }
    Ok(())
}

// 100644 blob 828a40d787d103ba6e31fa03d27738af8352b76d    .gitignore
// 040000 tree d4ebfcf79554803cf1a352a4328e962b84ce5c15    .vscode
// 100644 blob 97b86e422b1671807ae4dd0a9b8e270395461c48    Cargo.lock
// 100644 blob 465aaf991e6cb0eaaf3a55fa9d874a8d14596480    Cargo.toml
// 100644 blob 88bcb8ac5ad89ef48298e70cd7582eafb7207e1a    README.md
// 100644 blob ba2da70c710200e8f893b4ab44514c7ec9ce3621    diagrama.png
// 120000 blob bdd88d3f1e5e27dc6a4151dab6a0fbf9b03e0e97    gir
// 040000 tree 256faa745a0820774e9c16be60215fc27d4f6bd0    src
// 040000 tree d1bd5884df89a9734e3b0a4e7721a4802d85cce8    test_dir
// 100644 blob 678e12dc5c03a7cf6e9f64e688868962ab5d8b65    test_file.txt
// 100644 blob bdf08de0f3095da5030fecd9bafc0b00c1aced7c    test_file2.txt



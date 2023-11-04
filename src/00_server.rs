use gir::server::Servidor;
use std::env::args;
static SERVER_ARGS: usize = 2;
fn main() -> Result<(), ()> {
    let argv = args().collect::<Vec<String>>();
    if argv.len() != SERVER_ARGS {
        println!("Cantidad de argumentos inválido");
        let app_name = &argv[0];
        println!("Usage:\n{:?} <puerto>", app_name);
        return Err(());
    }

    let address = "127.0.0.1:".to_owned() + &argv[1];
    let mut sv = Servidor::new(&address).unwrap();
    sv.server_run().unwrap();
    Ok(())
}


// use gir::server::Servidor;
// fn main() -> Result<(), ()> {
//     Servidor::iniciar_servidor().unwrap();
//     Ok(())
// }

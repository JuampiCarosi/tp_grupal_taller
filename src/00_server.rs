use gir::server::Servidor;
fn main() -> Result<(), ()> {
    Servidor::iniciar_servidor().unwrap();
    Ok(())
}

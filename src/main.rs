use std::env::args;
use std::{
    io::{stdin, stdout, Write},
    net::TcpStream,
    path::PathBuf,
    sync::Arc,
};
static CLIENT_ARGS: usize = 2;

use gir::{
    tipos_de_dato::{comando::Comando, comunicacion::Comunicacion, logger::Logger},
    utils::{
        gir_config::{conseguir_ubicacion_log_config},
    },
};

fn interpretar_input(input: &String) -> Vec<String> {
    let mut result = Vec::new();
    let mut palabra = String::new();
    let mut entre_comillas = false;

    for c in input.chars() {
        match c {
            ' ' if !entre_comillas => {
                if !palabra.is_empty() {
                    result.push(palabra.clone());
                    palabra.clear();
                }
            }
            '"' => {
                entre_comillas = !entre_comillas;
                if !palabra.is_empty() {
                    result.push(palabra.clone());
                    palabra.clear();
                }
            }
            '\n' => continue,
            any => palabra.push(any),
        }
    }

    if !palabra.is_empty() {
        result.push(palabra);
    }

    result
}

fn pedir_comando() -> Result<Vec<String>, String> {
    print!("> ");
    stdout().flush().unwrap();

    let mut buf = String::new();
    let _ = stdin()
        .read_line(&mut buf)
        .map_err(|e| format!("Error al leer la entrada\n{}", e))?;
    Ok(interpretar_input(&buf))
}

fn main() -> Result<(), String> {
    let logger = Arc::new(Logger::new(PathBuf::from(
        conseguir_ubicacion_log_config()?
    ))?);
    println!("\n Gir Iniciado\n");
    println!("Ingrese un comando:\n");

    let argv = args().collect::<Vec<String>>();
    if argv.len() != CLIENT_ARGS {
        println!("Cantidad de argumentos inválido");
        let app_name = &argv[0];
        println!("Usage:\n{:?} <puerto>", app_name);
        return Err(
            "Cantidad de argumentos inválido, forma de llamar: cargo run --bin client <puerto>"
                .to_string(),
        );
    }

    if argv[1] == "9418" {
        println!("Conectado a git-daemon");
        loop {
            let comunicacion = Arc::new(Comunicacion::<TcpStream>::new_desde_direccion_servidor(
                "127.0.0.1:9418",
            )?);
            let input = pedir_comando()?;

            if input[0] == "exit" {
                break;
            }

            if input[0] == "gui" {
                println!("Iniciando GUI...");
                gir::gui::ejecutar(logger.clone(), comunicacion.clone());
                continue;
            }

            let mut comando = match Comando::new(input, logger.clone(), comunicacion.clone()) {
                Ok(comando) => comando,
                Err(err) => {
                    println!("ERROR: {}\n", err);
                    logger.log(err);
                    continue;
                }
            };

            match comando.ejecutar() {
                Ok(mensaje) => {
                    println!("{}", mensaje.clone());
                    logger.log(mensaje);
                }
                Err(mensaje) => {
                    println!("ERROR: {}", mensaje);
                    logger.log(mensaje);
                }
            };
        }
    } else {
        println!("Conectado a servidor de gir con el puerto: {}", &argv[1]);
        let comunicacion = Arc::new(Comunicacion::<TcpStream>::new_desde_direccion_servidor(
            &("127.0.0.1:".to_string() + &argv[1]),
        )?);
        loop {
            let input = pedir_comando()?;

            if input[0] == "exit" {
                break;
            }

            if input[0] == "gui" {
                println!("Iniciando GUI...");
                gir::gui::ejecutar(logger.clone(), comunicacion.clone());
                continue;
            }

            let mut comando = match Comando::new(input, logger.clone(), comunicacion.clone()) {
                Ok(comando) => comando,
                Err(err) => {
                    println!("ERROR: {}\n", err);
                    logger.log(err);
                    continue;
                }
            };

            match comando.ejecutar() {
                Ok(mensaje) => {
                    println!("{}", mensaje.clone());
                    logger.log(mensaje);
                }
                Err(mensaje) => {
                    println!("ERROR: {}", mensaje);
                    logger.log(mensaje);
                }
            };
        }
    }
    Ok(())
}

use std::{
    io::{stdin, stdout, Write},
    net::TcpStream,
    path::PathBuf,
    sync::Arc,
};

use gir::{
    tipos_de_dato::{comando::Comando, comunicacion::Comunicacion, logger::Logger},
    utils::{gir_config::obtener_gir_config_path, io::leer_a_string},
};

//extrae la ubiacion del archivo log seteada en el archivo config. En caso de error
// devuelve una direccion default = .log
fn obtener_dir_archivo_log() -> Result<String, String> {
    let ubicacion_config = obtener_gir_config_path()?;

    let mut dir_archivo_log = ".log".to_string();

    let contenido_config = match leer_a_string(ubicacion_config) {
        Ok(contenido_config) => contenido_config,
        Err(_) => return Ok(dir_archivo_log),
    };

    for linea_config in contenido_config.lines() {
        if linea_config.trim().starts_with("log") {
            if let Some(dir_archivo_log_config) = linea_config.split('=').nth(1) {
                dir_archivo_log = dir_archivo_log_config.trim().to_string();
                break;
            }
        }
    }

    Ok(dir_archivo_log)
}

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
    let logger = Arc::new(Logger::new(PathBuf::from(obtener_dir_archivo_log()?))?);
    println!("\n Gir Iniciado\n");
    println!("Ingrese un comando:\n");

    let mut comunicacion = Arc::new(Comunicacion::<TcpStream>::new_desde_direccion_servidor(
        "127.0.0.1:9418",
    )?);

    loop {
        let input = pedir_comando()?;

        if input[0] == "exit" {
            break;
        }

        if input[0] == "gui" {
            println!("Iniciando GUI...");
            gir::gui::ejecutar(logger.clone());
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
    Ok(())
}

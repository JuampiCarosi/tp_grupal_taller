use std::path::PathBuf;

use super::io;

pub fn obtener_gir_config_path() -> Result<String, String> {
    let home = std::env::var("HOME").map_err(|_| "Error al obtener el directorio home")?;
    let config_path = format!("{home}/.girconfig");
    Ok(config_path)
}

pub fn conseguir_nombre_y_mail_del_config() -> Result<(String, String), String> {
    let nombre = conseguir_nombre_config()?;
    let mail = conseguir_mail_config()?;

    Ok((nombre, mail))
}

///extrae el nombre seteada en el archivo config.
///Busca una entrada que sea nombre:
pub fn conseguir_nombre_config() -> Result<String, String> {
    buscar_en_config_el_valor_de("nombre")
}

///extrae el mail seteada en el archivo config.
///Busca una entrada que sea mail=
pub fn conseguir_mail_config() -> Result<String, String> {
    buscar_en_config_el_valor_de("mail")
}

//extrae la ubiacion del archivo log seteada en el archivo config. En caso de error
// devuelve una direccion default = .log
pub fn conseguir_ubicacion_log_config() -> Result<PathBuf, String> {
    let mut dir_archivo_log = PathBuf::from(".log");
    let ubicacion_config = obtener_gir_config_path()?;
    let contenido_config = match io::leer_a_string(ubicacion_config) {
        Ok(contenido_config) => contenido_config,
        Err(_) => return Ok(dir_archivo_log),
    };

    for linea_config in contenido_config.lines() {
        if linea_config.trim().starts_with("log") {
            if let Some(dir_archivo_log_config) = linea_config.split('=').nth(1) {
                dir_archivo_log = PathBuf::from(dir_archivo_log_config.trim());
                break;
            }
        }
    }

    Ok(dir_archivo_log)
}
///extrae el ip:puerto seteada en el archivo config.
///Busca una entrada que sea 'ip:puerto='
pub fn conseguir_direccion_ip_y_puerto() -> Result<String, String> {
    buscar_en_config_el_valor_de("ip:puerto")
}

///extrae el remoto seteada en el archivo config.
///Busca una entrada que sea 'remoto='
pub fn conseguir_direccion_nombre_remoto() -> Result<String, String> {
    buscar_en_config_el_valor_de("remoto")
}

///extrae el repositorio seteada en el archivo config.
///Busca una entrada que sea 'repositorio='
pub fn conseguir_direccion_nombre_repositorio() -> Result<String, String> {
    buscar_en_config_el_valor_de("repositorio")
}

fn buscar_en_config_el_valor_de(parametro_a_buscar: &str) -> Result<String, String> {
    let config_path = obtener_gir_config_path()?;
    let contenido_config = io::leer_a_string(config_path)?;

    for linea_config in contenido_config.lines() {
        if linea_config.trim().starts_with(&parametro_a_buscar) {
            if let Some(repositorio) = linea_config.split('=').nth(1) {
                return Ok(repositorio.trim().to_string());
            }
        }
    }

    Err(format!(
        "No se encontro {} del usario en config",
        parametro_a_buscar
    ))
}

fn archivo_config_esta_vacio() -> Result<bool, String> {
    let config_path = obtener_gir_config_path()?;

    let contenido = match io::leer_a_string(config_path) {
        Ok(contenido) => contenido,
        Err(_) => return Ok(true),
    };
    if contenido.is_empty() {
        return Ok(true);
    }
    Ok(false)
}

pub fn armar_config_con_mail_y_nombre() -> Result<(), String> {
    if !archivo_config_esta_vacio()? {
        return Ok(());
    }
    let mut nombre = String::new();
    let mut mail = String::new();

    println!("Por favor, ingrese su nombre:");
    match std::io::stdin().read_line(&mut nombre) {
        Ok(_) => (),
        Err(_) => return Err("No se pudo leer el nombre ingresado".to_string()),
    };

    println!("Por favor, ingrese su correo electrónico:");
    match std::io::stdin().read_line(&mut mail) {
        Ok(_) => (),
        Err(_) => return Err("No se pudo leer el mail ingresado".to_string()),
    };

    nombre = nombre.trim().to_string();
    mail = mail.trim().to_string();

    let config_path = obtener_gir_config_path()?;
    let contenido = format!("nombre ={}\nmail ={}\n", nombre, mail);
    io::escribir_bytes(config_path, contenido)?;
    println!("Información de usuario guardada en ~/.girconfig.");
    Ok(())
}

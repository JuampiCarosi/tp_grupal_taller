use crate::io::{escribir_bytes, leer_a_string};

pub fn obtener_gir_config_path() -> Result<String, String> {
    let home = std::env::var("HOME").map_err(|_| "Error al obtener el directorio home")?;
    let config_path = format!("{home}/.girconfig");
    Ok(config_path)
}

pub fn conseguir_nombre_y_mail_del_config() -> Result<(String, String), String> {
    let config_path = obtener_gir_config_path()?;
    let contenido = leer_a_string(config_path)?;

    let lineas = contenido.split('\n').collect::<Vec<&str>>();
    let nombre = lineas[0].split('=').collect::<Vec<&str>>()[1].trim();
    let mail = lineas[1].split('=').collect::<Vec<&str>>()[1].trim();
    Ok((nombre.to_string(), mail.to_string()))
}

fn archivo_config_esta_vacio() -> Result<bool, String> {
    let config_path = obtener_gir_config_path()?;

    let contenido = match leer_a_string(config_path) {
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
    escribir_bytes(config_path, contenido)?;
    println!("Información de usuario guardada en ~/.girconfig.");
    Ok(())
}

// pub fn eliminar_prefijos(lineas: &mut Vec<String>, prefijo: &str) -> Vec<String> {
//     let mut lineas_sin_prefijo: Vec<String> = Vec::new();
//     for linea in lineas {
//         lineas_sin_prefijo.push(linea.drain(prefijo.len()..linea.len()).as_str().to_string());

//     }
//     lineas_sin_prefijo
// }

pub fn eliminar_prefijos(lineas: &Vec<String>) -> Vec<String> {
    let mut lineas_sin_prefijo: Vec<String> = Vec::new();
    for linea in lineas {
        lineas_sin_prefijo.push(linea.split_whitespace().collect::<Vec<&str>>()[1].to_string())
    }
    lineas_sin_prefijo
}

///Obtiene de la url el ip puerto y el repositorio
///
/// ## Ejemplo
/// - recibe: ip:puerto/repositorio/
/// - devuelve: (ip:puerto, /respositorio/)
pub fn obtener_ip_puerto_y_repositorio(url: &str) -> Result<(String, String), String> {
    let (ip_puerto_str, repositorio) = url
        .split_once("/")
        .ok_or_else(|| format!("Fallo en obtener el ip:puerto y repo de {}", url))?;

    Ok((ip_puerto_str.to_string(), "/".to_string() + repositorio))
}

/// Elimina el prefijo de las lineas. Por ejemplo, si el prefijo es "remote = ", y la linea es "remote = origin", devuelve "origin".
pub fn eliminar_prefijos(lineas: &Vec<String>) -> Vec<String> {
    let mut lineas_sin_prefijo: Vec<String> = Vec::new();
    for linea in lineas {
        lineas_sin_prefijo.push(linea.split_whitespace().collect::<Vec<&str>>()[1].to_string())
    }
    lineas_sin_prefijo
}

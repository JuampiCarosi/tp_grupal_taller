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
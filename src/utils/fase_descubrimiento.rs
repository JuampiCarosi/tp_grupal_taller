use std::{
    io::{Read, Write},
    path::PathBuf,
};

use crate::tipos_de_dato::comunicacion::Comunicacion;

use super::ramas;

///Se encarga de la fase de descubrimiento con el servidor, en la cual se recibe del servidor
/// una lista de referencias.
/// La primera linea contiene la version del server
/// La segunda linea recibida tiene el siguiente : 'hash_del_commit_head HEAD'\0'lista de capacida'
/// Las siguients lineas: 'hash_del_commit_cabeza_de_rama_en_el_servidor'
///                        'direccion de la carpeta de la rama en el servidor'
///
/// # Resultado
/// - vector con las capacidades del servidor
/// - hash del commit cabeza de rama
/// - vector de tuplas con los hash del commit cabeza de rama y la direccion de la
///     carpeta de la rama en el servidor(ojo!! la direccion para el servidor no para el local)
/// - vector de tuplas con el hash del commit y el tag asosiado
pub fn fase_de_descubrimiento<T: Write + Read>(
    comunicacion: Comunicacion<T>,
) -> Result<
    (
        Vec<String>,
        Option<String>,
        Vec<(String, PathBuf)>,
        Vec<(String, PathBuf)>,
    ),
    String,
> {
    let mut lineas_recibidas = comunicacion.obtener_lineas()?;
    println!("Se recibio {:?}\n", lineas_recibidas);
    let _version = lineas_recibidas.remove(0); //la version del server

    let segunda_linea = lineas_recibidas.remove(0);

    let (contenido, capacidades) = separara_capacidades(&segunda_linea)?;
    let commit_head_remoto = separar_commit_head_de_ser_necesario(contenido, &mut lineas_recibidas);

    let (commits_cabezas_y_dir_rama_asosiado, commits_y_tags_asosiados) =
        obtener_commits_y_dir_rama_o_tag_asosiados(&lineas_recibidas)?;

    Ok((
        capacidades,
        commit_head_remoto,
        commits_cabezas_y_dir_rama_asosiado,
        commits_y_tags_asosiados,
    ))
}

fn separara_capacidades(primera_linea: &String) -> Result<(String, Vec<String>), String> {
    let (contenido, capacidades) = primera_linea
        .split_once('\0')
        .ok_or("Fallo al separar la linea en commit y capacidades\n".to_string())?;

    let capacidades_vector: Vec<String> = capacidades
        .split_whitespace()
        .map(|s| s.to_string())
        .collect();

    Ok((contenido.to_string(), capacidades_vector))
}

fn separar_commit_head_de_ser_necesario(
    contenido: String,
    lineas_recibidas: &mut Vec<String>,
) -> Option<String> {
    let mut commit_head_remoto = Option::None;

    if contenido.contains("HEAD") {
        commit_head_remoto = Option::Some(contenido.replace("HEAD", "").trim().to_string());
    } else {
        lineas_recibidas.insert(0, contenido);
    }
    commit_head_remoto
}

fn obtener_commits_y_dir_rama_o_tag_asosiados(
    lineas_recibidas: &Vec<String>,
) -> Result<(Vec<(String, PathBuf)>, Vec<(String, PathBuf)>), String> {
    let mut commits_cabezas_y_dir_rama_asosiados: Vec<(String, PathBuf)> = Vec::new();

    let mut commits_y_tags_asosiados: Vec<(String, PathBuf)> = Vec::new();

    for linea in lineas_recibidas {
        let (commit, dir) = obtener_commit_y_dir_asosiado(linea)?;

        if ramas::es_la_ruta_a_una_rama(&dir) {
            commits_cabezas_y_dir_rama_asosiados.push((commit, dir));
        } else {
            commits_y_tags_asosiados.push((commit, dir));
        }
    }

    Ok((
        commits_cabezas_y_dir_rama_asosiados,
        commits_y_tags_asosiados,
    ))
}

///Separa el commit del dir asosiado
///
/// # argumento
///
/// referencia: un string con el commit y la rama o tag asosiado. Con el formato:
///     "'hash del commit' 'rama_remota/tag'"
fn obtener_commit_y_dir_asosiado(referencia: &String) -> Result<(String, PathBuf), String> {
    let (commit_cabeza_de_rama, dir) = referencia
        .split_once(' ')
        .ok_or("Fallo al separar el conendio en actualizar referencias\n".to_string())?;

    let dir_path = PathBuf::from(dir.trim());
    Ok((commit_cabeza_de_rama.to_string(), dir_path))
}

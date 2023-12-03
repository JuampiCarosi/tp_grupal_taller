use std::path::{Path, PathBuf};

use super::{io, path_buf};

///obtiene el nombre de la rama acutal
pub fn obtener_rama_actual() -> Result<String, String> {
    let dir_rama_actual = obtener_ref_rama_actual()?;
    let rama = path_buf::obtener_nombre(&dir_rama_actual)?;
    Ok(rama)
}

///obtiene el commit cabeza de rama de la rama actual
pub fn obtner_commit_head_rama_acutual() -> Result<String, String> {
    let dir = obtener_gir_dir_rama_actual()?;
    io::leer_a_string(dir)
}

///obtiene la ref de la rama actual
pub fn obtener_ref_rama_actual() -> Result<PathBuf, String> {
    let contenido_head = io::leer_a_string("./.gir/HEAD")?;
    let (_, dir_rama_actual) = contenido_head
        .split_once(' ')
        .ok_or("Fallo al obtener la rama actual\n".to_string())?;
    Ok(PathBuf::from(dir_rama_actual.trim()))
}

pub fn obtener_gir_dir_rama_actual() -> Result<PathBuf, String> {
    let ref_rama_actual = obtener_ref_rama_actual()?;
    print!("{}", ref_rama_actual.display());

    let dir_rama = PathBuf::from("./.gir").join(ref_rama_actual);
    // print!("{}", dir_rama.display());
    Ok(dir_rama)
}

///Comprueba si dir es el la ruta a una carpeta que corresponde a una rama o a una
/// tag.
///
/// Si el path contien heads entonces es una rama, devuelve true. Caso contrio es un tag,
/// devuelve false
pub fn es_la_ruta_a_una_rama(dir: &Path) -> bool {
    for componente in dir.iter() {
        if let Some(componente_str) = componente.to_str() {
            if componente_str == "heads" {
                return true;
            }
        }
    }
    false
}

/// Convierte una rama que el servidor la ve como local a una en la cual el cliente ve como remota
///
/// # Ejemplo:
///
/// recive:  ./.gir/refs/heads/master o refs/heads/master
/// devuelve: ./.gir/refs/remotes/{remoto}/master
pub fn convertir_de_dir_rama_remota_a_dir_rama_local(
    remoto: &str,
    dir_rama_remota: &Path,
) -> Result<PathBuf, String> {
    let carpeta_del_remoto = format!("./.gir/refs/remotes/{}/", remoto);

    let rama_remota = path_buf::obtener_nombre(dir_rama_remota)?;
    let dir_rama_local = PathBuf::from(carpeta_del_remoto + rama_remota.as_str());

    Ok(dir_rama_local)
}

///Verificar si la rama remota existe, devuelve true. Caso contrario false
///
/// ## Argumentos
/// - rama_remota: semi path a la rama remota(Ej: origin/aaaa)
pub fn existe_la_rama_remota(rama_remota: &str) -> bool {
    let dir_rama_remota = PathBuf::from(format!("./.gir/refs/remotes/{}", rama_remota));

    dir_rama_remota.exists()
}

///Verificar si la rama existe, devuelve true. Caso contrario false
///
/// ## Argumentos
/// - rama: nombre de la rama(Ej: aaaa)
pub fn existe_la_rama(rama: &str) -> bool {
    let dir_rama = PathBuf::from(format!("./.gir/refs/heads/{}", rama));

    dir_rama.exists()
}

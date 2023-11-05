use std::path::{Path, PathBuf};

pub fn esta_directorio_habilitado(
    directorio: &Path,
    directorios_habilitados: &Vec<PathBuf>,
) -> bool {
    for directorio_habilitado in directorios_habilitados {
        if directorio.starts_with(directorio_habilitado)
            || directorio_habilitado.starts_with(directorio)
        {
            return true;
        }
    }
    false
}

pub fn obtener_directorio_raiz(directorio: &Path) -> Result<String, String> {
    let directorio_split = directorio
        .iter()
        .next()
        .ok_or("Error al obtener el directorio raiz")?
        .to_str()
        .ok_or("Error al obtener el directorio raiz")?;

    Ok(directorio_split.to_string())
}

pub fn obtener_nombre(directorio: &Path) -> Result<String, String> {
    let directorio_split = directorio
        .file_name()
        .ok_or("Error al obtener el nombre")?
        .to_str()
        .ok_or("Error al obtener el nombre")?;

    Ok(directorio_split.to_string())
}

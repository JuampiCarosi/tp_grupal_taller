use std::path::PathBuf;

pub fn esta_directorio_habilitado(
    directorio: &PathBuf,
    directorios_habilitados: &Vec<PathBuf>,
) -> bool {
    for directorio_habilitado in directorios_habilitados {
        if directorio.starts_with(directorio_habilitado)
            || directorio_habilitado.starts_with(directorio)
        {
            return true;
        }
    }
    return false;
}

pub fn obtener_directorio_raiz(directorio: &PathBuf) -> Result<String, String> {
    let directorio_split = directorio
        .into_iter()
        .next()
        .ok_or_else(|| "Error al obtener el directorio raiz")?
        .to_str()
        .ok_or_else(|| "Error al obtener el directorio raiz")?;

    Ok(directorio_split.to_string())
}

pub fn obtener_nombre(directorio: &PathBuf) -> Result<String, String> {
    let directorio_split = directorio
        .file_name()
        .ok_or_else(|| "Error al obtener el directorio raiz")?
        .to_str()
        .ok_or_else(|| "Error al obtener el directorio raiz")?;

    Ok(directorio_split.to_string())
}

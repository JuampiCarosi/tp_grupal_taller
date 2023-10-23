use std::{
    fs::{self, File},
    path::Path,
};

pub fn crear_directorio<P: AsRef<Path> + Clone>(directorio: P) -> Result<(), String> {
    let dir = fs::metadata(directorio.clone());
    if dir.is_ok() {
        return Ok(());
    }
    match fs::create_dir_all(directorio) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("Error al crear el directorio: {}", e)),
    }
}

pub fn crear_archivo<P: AsRef<Path> + Clone>(dir_directorio: P) -> Result<(), String> {
    si_no_existe_directorio_de_archivo_crearlo(&dir_directorio)?;
    if !dir_directorio.as_ref().exists() {
        File::create(dir_directorio.clone()).map_err(|err| format!("{}", err))?;
    }

    Ok(())
}

pub fn leer_a_string<P>(path: P) -> Result<String, String>
where
    P: AsRef<Path>,
{
    match fs::read_to_string(&path) {
        Ok(contenido) => Ok(contenido),
        Err(_) => Err(format!(
            "No se pudo leer el archivo {}",
            path.as_ref().display()
        )),
    }
}

pub fn escribir_bytes<P, C>(dir_archivo: P, contenido: C) -> Result<(), String>
where
    P: AsRef<Path>,
    C: AsRef<[u8]>,
{
    si_no_existe_directorio_de_archivo_crearlo(&dir_archivo)?;

    match fs::write(dir_archivo, contenido) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("Error al escribir el archivo: {}", e)),
    }
}

pub fn leer_bytes<P>(archivo: P) -> Result<Vec<u8>, String>
where
    P: AsRef<Path>,
{
    match fs::read(&archivo) {
        Ok(contenido) => Ok(contenido),
        Err(_) => Err(format!(
            "No se pudo leer el archivo {}",
            archivo.as_ref().display()
        )),
    }
}
fn si_no_existe_directorio_de_archivo_crearlo<P>(dir_archivo: &P) -> Result<(), String>
where
    P: AsRef<Path>,
{
    let dir = dir_archivo.as_ref().parent();
    if let Some(parent_dir) = dir {
        let parent_str = parent_dir
            .to_str()
            .ok_or_else(|| String::from("Error al convertir el directorio a cadena"))?;

        crear_directorio(parent_str.to_owned() + "/")?;
    };
    Ok(())
}

pub fn rm_directorio<P>(directorio: P) -> Result<(), String>
where
    P: AsRef<Path>,
{
    let metadata = fs::metadata(&directorio).map_err(|_| {
        format!(
            "No se pudo borrar el directorio {}",
            directorio.as_ref().display()
        )
    })?;

    if metadata.is_file() {
        return match fs::remove_file(&directorio) {
            Ok(_) => Ok(()),
            Err(_) => Err(format!(
                "No se pudo borrar el directorio {}",
                directorio.as_ref().display()
            )),
        };
    }

    if metadata.is_dir() {
        return match fs::remove_dir_all(&directorio) {
            Ok(_) => Ok(()),
            Err(_) => Err(format!(
                "No se pudo borrar el directorio {}",
                directorio.as_ref().display()
            )),
        };
    }
    Err(format!(
        "No se pudo borrar el directorio {}",
        directorio.as_ref().display()
    ))
}

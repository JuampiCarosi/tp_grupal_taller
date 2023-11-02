fn obtener_gir_config_path() -> Result<String, String> {
    let home = std::env::var("HOME").map_err(|_| "Error al obtener el directorio home")?;
    let config_path = format!("{home}/.girconfig");
    Ok(config_path)
}

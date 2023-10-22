pub enum Visualizaciones {
    TipoObjeto,
    Tamanio,
    Contenido,
}

impl Visualizaciones {
    pub fn from(parametro: String) -> Result<Visualizaciones, String> {
        match parametro.as_str() {
            "-t" => Ok(Visualizaciones::TipoObjeto),
            "-s" => Ok(Visualizaciones::Tamanio),
            "-p" => Ok(Visualizaciones::Contenido),
            _ => Err(format!(
                "Parametro desconocido {}, parametros esperados: (-t | -s | -p)",
                parametro
            )),
        }
    }
}

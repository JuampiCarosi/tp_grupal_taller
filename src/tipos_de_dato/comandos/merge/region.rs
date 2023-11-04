#[derive(Clone)]
pub enum Region {
    Normal(String),
    Conflicto(String, String),
}

impl std::fmt::Debug for Region {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Region::Normal(contenido) => write!(f, "Normal({})", contenido),
            Region::Conflicto(contenido_head, contenido_entrante) => {
                write!(f, "Conflicto({},{})", contenido_head, contenido_entrante)
            }
        }
    }
}
impl std::fmt::Display for Region {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Region::Normal(contenido) => write!(f, "{}", contenido),
            Region::Conflicto(contenido_head, contenido_entrante) => {
                write!(
                    f,
                    "<<<<<< HEAD\n{}\n======\n{}\n>>>>>> Entrante\n)",
                    contenido_head, contenido_entrante
                )
            }
        }
    }
}

pub fn unificar_regiones(regiones: Vec<Region>) -> Vec<Region> {
    let mut regiones_unificadas: Vec<Region> = Vec::new();
    let mut i = 0;
    while i < regiones.len() {
        match &regiones[i] {
            Region::Normal(_) => {
                regiones_unificadas.push(regiones[i].clone());
                i += 1;
            }
            Region::Conflicto(_, _) => {
                let mut j = i;
                let mut buffer_head = String::new();
                let mut buffer_entrante = String::new();
                while j < regiones.len() {
                    match &regiones[j] {
                        Region::Normal(_) => break,
                        Region::Conflicto(lado_head, lado_entrante) => {
                            buffer_head.push_str(&lado_head);
                            buffer_entrante.push_str(&lado_entrante);
                        }
                    }
                    j += 1;
                }
                regiones_unificadas.push(Region::Conflicto(buffer_head, buffer_entrante));
                i = j;
            }
        }
    }
    regiones_unificadas
}

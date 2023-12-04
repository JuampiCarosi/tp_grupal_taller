#[derive(Clone, Debug)]

/// Estructura que representa una fecha de un commit.
pub struct Date {
    /// Fecha del commit, guardada en fomarto unix.
    pub tiempo: String,
    /// Offset de la fecha del commit.
    pub offset: String,
}

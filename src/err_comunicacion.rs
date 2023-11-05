use std::{io, str::Utf8Error};

#[derive(Debug)]
pub enum   ErrorDeComunicacion {
    Utf8Error(Utf8Error),
    IoError(io::Error),
}

impl From<Utf8Error> for ErrorDeComunicacion {
    fn from(error: Utf8Error) -> Self {
        ErrorDeComunicacion::Utf8Error(error)
    }
}

impl From<io::Error> for ErrorDeComunicacion {
    fn from(error: io::Error) -> Self {
        ErrorDeComunicacion::IoError(error)
    }
}

use std::{net::TcpStream, sync::Arc};

use gtk::prelude::*;

use crate::tipos_de_dato::{comunicacion::Comunicacion, logger::Logger};

use super::hidratar_componentes;

pub fn render(
    builder: &gtk::Builder,
    window: &gtk::Window,
    logger: Arc<Logger>,
    branch_actual: String,
    comunicacion: Arc<Comunicacion<TcpStream>>,
) {
    let icon = builder.object::<gtk::EventBox>("refresh-icon").unwrap();

    let builder = builder.clone();
    let window = window.clone();
    icon.connect_button_press_event(move |_, _| {
        hidratar_componentes(
            &builder,
            &window,
            logger.clone(),
            branch_actual.clone(),
            comunicacion.clone(),
        );
        gtk::glib::Propagation::Proceed
    });
}

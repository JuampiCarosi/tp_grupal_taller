use gtk::{self};
use gtk::{prelude::*, Spinner};
use std::net::TcpStream;
use std::sync::Arc;
use std::thread::sleep;

use crate::tipos_de_dato::comandos::branch;
use crate::tipos_de_dato::comandos::pull::Pull;
use crate::tipos_de_dato::comunicacion::Comunicacion;

use super::hidratar_componentes;

pub fn render(
    builder: &gtk::Builder,
    window: &gtk::Window,
    comunicacion: Arc<Comunicacion<TcpStream>>,
    logger: Arc<crate::tipos_de_dato::logger::Logger>,
    branch_actual: String,
) {
    let pull_button = builder.object::<gtk::Button>("pull-button").unwrap();

    let builder_clone = builder.clone();
    let window_clone = window.clone();
    pull_button.connect_clicked(move |_| {
        let fetching_dialog = builder_clone
            .object::<gtk::Dialog>("fetching-dialog")
            .unwrap();

        fetching_dialog.set_position(gtk::WindowPosition::Center);
        fetching_dialog.show_all();
        Pull::from(logger.clone(), comunicacion.clone())
            .unwrap()
            .ejecutar()
            .unwrap();

        hidratar_componentes(
            &builder_clone,
            &window_clone,
            logger.clone(),
            branch_actual.clone(),
            comunicacion.clone(),
        );

        sleep(std::time::Duration::from_secs(3));
        fetching_dialog.close();
    });
}

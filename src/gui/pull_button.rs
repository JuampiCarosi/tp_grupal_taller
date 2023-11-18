use gtk::prelude::*;
use gtk::{self};
use std::sync::Arc;
use std::thread::sleep;

use crate::tipos_de_dato::comandos::pull::Pull;

use super::{error_dialog, hidratar_componentes};

pub fn render(
    builder: &gtk::Builder,
    window: &gtk::Window,
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

        match Pull::from(logger.clone()).unwrap().ejecutar() {
            Ok(_) => {}
            Err(err) => {
                error_dialog::mostrar_error(&err);
                return;
            }
        };

        hidratar_componentes(
            &builder_clone,
            &window_clone,
            logger.clone(),
            branch_actual.clone(),
        );

        sleep(std::time::Duration::from_secs(3));
        fetching_dialog.close();
    });
}

use super::comando_gui::ComandoGui;
use super::hidratar_componentes;
use crate::tipos_de_dato::comandos::pull::Pull;
use gtk::prelude::*;
use gtk::{self};
use std::sync::Arc;
use std::thread::sleep;

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

        if let None = Pull::from(Vec::new(), logger.clone()).ejecutar_gui() {
            fetching_dialog.close();
            return;
        }

        hidratar_componentes(
            &builder_clone,
            &window_clone,
            logger.clone(),
            &branch_actual,
        );

        sleep(std::time::Duration::from_secs(3));
        fetching_dialog.close();
    });
}

use gtk::glib::Propagation;
use gtk::prelude::*;
use gtk::{self};
use std::sync::Arc;

use crate::gui::comando_gui::ComandoGui;
use crate::tipos_de_dato::comandos::push::Push;
use crate::tipos_de_dato::logger::Logger;

pub fn render(builder: &gtk::Builder, _window: &gtk::Window, logger: Arc<Logger>) {
    let push_button = builder.object::<gtk::Button>("push-button").unwrap();

    let builder_clone = builder.clone();
    push_button.connect_clicked(move |_| {
        let fetching_dialog = builder_clone
            .object::<gtk::MessageDialog>("fetching-dialog")
            .unwrap();

        fetching_dialog.set_position(gtk::WindowPosition::Center);

        let logger_clone = logger.clone();
        let id = fetching_dialog.connect_focus_in_event(move |dialog, _| {
            Push::new(&mut Vec::new(), logger_clone.clone()).ejecutar_gui();
            dialog.hide();
            Propagation::Stop
        });

        fetching_dialog.run();
        fetching_dialog.disconnect(id);
    });
}

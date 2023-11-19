use gtk::prelude::*;
use gtk::{self};
use std::sync::Arc;
use std::thread::sleep;

use crate::tipos_de_dato::comandos::push::Push;
use crate::tipos_de_dato::logger::Logger;

use super::error_dialog;

pub fn render(builder: &gtk::Builder, _window: &gtk::Window, logger: Arc<Logger>) {
    let push_button = builder.object::<gtk::Button>("push-button").unwrap();

    let builder_clone = builder.clone();
    push_button.connect_clicked(move |_| {
        let fetching_dialog = builder_clone
            .object::<gtk::Dialog>("fetching-dialog")
            .unwrap();

        fetching_dialog.set_position(gtk::WindowPosition::Center);
        fetching_dialog.show_all();
        let mut push = match Push::new(logger.clone()) {
            Ok(push) => push,
            Err(err) => {
                error_dialog::mostrar_error(&err);
                return;
            }
        };

        match push.ejecutar() {
            Ok(_) => {}
            Err(err) => {
                error_dialog::mostrar_error(&err);
                return;
            }
        };
        sleep(std::time::Duration::from_secs(3));
        fetching_dialog.close();
    });
}

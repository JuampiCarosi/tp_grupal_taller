use gtk::{self};
use gtk::{prelude::*, Spinner};
use std::net::TcpStream;
use std::thread::sleep;

use crate::tipos_de_dato::comunicacion::Comunicacion;

pub fn render(
    builder: &gtk::Builder,
    window: &gtk::Window,
    // comunicacion: Comunicacion<TcpStream>
) {
    let push_button = builder.object::<gtk::Button>("push-button").unwrap();

    let window_clone = window.clone();
    let builder_clone = builder.clone();
    push_button.connect_clicked(move |_| {
        // Push::new(&mut comunicacion_clone).ejecutar();
        let fetching_dialog = builder_clone
            .object::<gtk::Dialog>("fetching-dialog")
            .unwrap();

        fetching_dialog.set_position(gtk::WindowPosition::Center);

        fetching_dialog.show_all();
        sleep(std::time::Duration::from_secs(1));
        fetching_dialog.close();
    });
}

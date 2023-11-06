use gtk::{self};
use gtk::{prelude::*, Spinner};
use std::net::TcpStream;
use std::sync::Arc;
use std::thread::sleep;

use crate::tipos_de_dato::comandos::push::Push;
use crate::tipos_de_dato::comunicacion::Comunicacion;

pub fn render(
    builder: &gtk::Builder,
    window: &gtk::Window,
    comunicacion: Arc<Comunicacion<TcpStream>>,
) {
    let push_button = builder.object::<gtk::Button>("push-button").unwrap();

    let builder_clone = builder.clone();
    let comunicacion_clone = comunicacion.clone();
    push_button.connect_clicked(move |_| {
        let fetching_dialog = builder_clone
            .object::<gtk::Dialog>("fetching-dialog")
            .unwrap();

        fetching_dialog.set_position(gtk::WindowPosition::Center);
        fetching_dialog.show_all();
        Push::new(comunicacion_clone.clone()).ejecutar().unwrap();
        sleep(std::time::Duration::from_secs(3));
        fetching_dialog.close();
    });
}

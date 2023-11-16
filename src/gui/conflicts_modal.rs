use std::sync::Arc;

use gtk::{prelude::*, Button};

use crate::tipos_de_dato::{comandos::merge::Merge, logger::Logger};

fn conflicts_button(builder: &gtk::Builder, logger: Arc<Logger>) {
    let boton: Button = builder.object("conflicts-button").unwrap();
    if !Merge::hay_archivos_sin_mergear(logger.clone()).unwrap() {
        boton.set_sensitive(false);
    }

    let builder = builder.clone();
    boton.connect_clicked(move |_| {
        modal(&builder);
    });
}

fn modal(builder: &gtk::Builder) {
    let modal: gtk::Window = builder.object("conflicts-window").unwrap();
    modal.show_all();
}

pub fn render(builder: &gtk::Builder, window: &gtk::Window, logger: Arc<Logger>) {
    conflicts_button(builder, logger.clone());
}

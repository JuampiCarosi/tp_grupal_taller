use std::sync::Arc;

use gtk::prelude::*;

use crate::tipos_de_dato::{comandos::clone::Clone, logger::Logger};

use super::branch_selector;

fn run_dialog(builder: &gtk::Builder) {
    let dialog: gtk::MessageDialog = builder.object("clone").unwrap();
    dialog.run();
    dialog.hide();
}

fn boton_confimar_dialog(builder: &gtk::Builder, window: &gtk::Window, logger: Arc<Logger>) {
    let confirm: gtk::Button = builder.object("confirm-clone").unwrap();
    let dialog: gtk::MessageDialog = builder.object("clone").unwrap();
    let input: gtk::Entry = builder.object("clone-input").unwrap();

    let builder_clone = builder.clone();
    let window_clone = window.clone();
    confirm.connect_clicked(move |_| {
        // Clone::from(logger.clone()).ejecutar().unwrap();
        println!("Clonando repositorio: {}", input.text());
        input.set_text("");
        dialog.hide();
    });
}

pub fn render(builder: &gtk::Builder, window: &gtk::Window, logger: Arc<Logger>) {
    run_dialog(builder);
    boton_confimar_dialog(builder, window, logger);
}

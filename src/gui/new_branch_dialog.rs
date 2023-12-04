use std::{path::PathBuf, sync::Arc};

use gtk::prelude::*;

use crate::tipos_de_dato::{comando::Ejecutar, comandos::branch::Branch, logger::Logger};

use super::{branch_selector, info_dialog};

fn run_dialog(builder: &gtk::Builder) {
    let branch_button: gtk::Button = builder.object("branch-button").unwrap();
    let dialog: gtk::MessageDialog = builder.object("branch").unwrap();

    dialog.set_position(gtk::WindowPosition::Center);

    branch_button.connect_clicked(move |_| {
        dialog.run();
        dialog.hide();
    });
}

fn boton_cancel_dialog(builder: &gtk::Builder) {
    let cancel: gtk::Button = builder.object("cancel-branch").unwrap();
    let dialog: gtk::MessageDialog = builder.object("branch").unwrap();

    cancel.connect_clicked(move |_| {
        dialog.hide();
    });
}

fn boton_confimar_dialog(builder: &gtk::Builder, window: &gtk::Window, logger: Arc<Logger>) {
    let confirm: gtk::Button = builder.object("confirm-branch").unwrap();
    let dialog: gtk::MessageDialog = builder.object("branch").unwrap();
    let input: gtk::Entry = builder.object("branch-input").unwrap();

    let builder_clone = builder.clone();
    let window_clone = window.clone();
    confirm.connect_clicked(move |_| {
        match Branch::from(
            &mut vec![input.text().to_string()],
            Arc::new(Logger::new(PathBuf::from("log.txt")).unwrap()),
        )
        .unwrap()
        .ejecutar()
        {
            Ok(_) => {}
            Err(err) => {
                info_dialog::mostrar_error(&err);
                return;
            }
        };

        branch_selector::render(&builder_clone, &window_clone, logger.clone());
        input.set_text("");
        dialog.hide();
    });
}

pub fn render(builder: &gtk::Builder, window: &gtk::Window, logger: Arc<Logger>) {
    run_dialog(builder);

    boton_cancel_dialog(builder);
    boton_confimar_dialog(builder, window, logger);
}

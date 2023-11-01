use std::{path::PathBuf, rc::Rc};

use gtk::prelude::*;

use crate::tipos_de_dato::{comandos::branch::Branch, logger::Logger};

use super::branch_selector;

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

fn boton_confimar_dialog(builder: &gtk::Builder, window: &gtk::Window) {
    let confirm: gtk::Button = builder.object("confirm-branch").unwrap();
    let dialog: gtk::MessageDialog = builder.object("branch").unwrap();
    let input: gtk::Entry = builder.object("branch-input").unwrap();

    let builder_clone = builder.clone();
    let window_clone = window.clone();
    confirm.connect_clicked(move |_| {
        Branch::from(
            &mut vec![input.text().to_string()],
            Rc::new(Logger::new(PathBuf::from("log.txt")).unwrap()),
        )
        .unwrap()
        .ejecutar()
        .unwrap();

        branch_selector::render(&builder_clone, &window_clone);
        input.set_text("");
        dialog.hide();
    });
}

pub fn render(builder: &gtk::Builder, window: &gtk::Window) {
    run_dialog(&builder);

    boton_cancel_dialog(&builder);
    boton_confimar_dialog(&builder, &window);
}

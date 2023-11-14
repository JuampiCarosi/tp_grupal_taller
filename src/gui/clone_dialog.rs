use std::{net::TcpStream, path::PathBuf, sync::Arc};

use gtk::prelude::*;

use crate::tipos_de_dato::{
    comandos::clone::Clone,
    comunicacion::{Comunicacion},
    logger::Logger,
};

use super::error_dialog;

fn run_dialog(builder: &gtk::Builder) {
    let dialog: gtk::MessageDialog = builder.object("clone").unwrap();
    dialog.run();
    dialog.hide();
}

fn clonar_dialog(
    builder: &gtk::Builder,
    logger: Arc<Logger>,
    comunicacion: Arc<Comunicacion<TcpStream>>,
) {
    let confirm: gtk::Button = builder.object("confirm-clone").unwrap();
    let dialog: gtk::MessageDialog = builder.object("clone").unwrap();
    let input: gtk::Entry = builder.object("clone-input").unwrap();
    dialog.set_position(gtk::WindowPosition::Center);

    confirm.connect_clicked(move |_| {
        match Clone::from(logger.clone(), comunicacion.clone()).ejecutar() {
            Ok(_) => {}
            Err(err) => error_dialog::mostrar_error(&err),
        }

        input.set_text("");
        dialog.hide();
    });
}

fn error_no_repo_dialog(builder: &gtk::Builder) {
    let dialog: gtk::MessageDialog = builder.object("no-repo-dialog").unwrap();
    let aceptar_button: gtk::Button = builder.object("no-repo-close").unwrap();
    dialog.set_position(gtk::WindowPosition::Center);

    let dialog_clone = dialog.clone();
    aceptar_button.connect_clicked(move |_| {
        dialog_clone.hide();
    });

    dialog.run();
}

pub fn render(
    builder: &gtk::Builder,
    logger: Arc<Logger>,
    comunicacion: Arc<Comunicacion<TcpStream>>,
) -> bool {
    clonar_dialog(builder, logger, comunicacion);
    run_dialog(builder);

    if !PathBuf::from(".gir").is_dir() {
        error_no_repo_dialog(builder);
        return false;
    }

    true
}

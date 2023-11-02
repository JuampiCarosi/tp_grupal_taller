use std::{path::PathBuf, sync::Arc};

use gtk::prelude::*;

use crate::tipos_de_dato::{
    comandos::commit::Commit,
    logger::{self, Logger},
};

use super::{log_list, staging_area};

fn run_dialog(builder: &gtk::Builder) {
    let commit_button: gtk::Button = builder.object("commit-button").unwrap();
    let dialog: gtk::MessageDialog = builder.object("commit").unwrap();

    dialog.set_position(gtk::WindowPosition::Center);

    commit_button.connect_clicked(move |_| {
        dialog.run();
        dialog.hide();
    });
}

fn boton_cancel_dialog(builder: &gtk::Builder) {
    let cancel: gtk::Button = builder.object("cancel-commit").unwrap();
    let dialog: gtk::MessageDialog = builder.object("commit").unwrap();

    cancel.connect_clicked(move |_| {
        dialog.hide();
    });
}

fn boton_confimar_dialog(builder: &gtk::Builder, window: &gtk::Window, logger: Arc<Logger>) {
    let confirm: gtk::Button = builder.object("confirm-commit").unwrap();
    let dialog: gtk::MessageDialog = builder.object("commit").unwrap();
    let input: gtk::Entry = builder.object("commit-input").unwrap();

    let builder_clone = builder.clone();
    let window_clone = window.clone();
    confirm.connect_clicked(move |_| {
        Commit::from(
            &mut vec!["-m".to_string(), input.text().to_string()],
            logger.clone(),
        )
        .unwrap()
        .ejecutar()
        .unwrap();

        let branch_actual = Commit::obtener_branch_actual().unwrap();

        log_list::render(&builder_clone, branch_actual);
        staging_area::render(&builder_clone, logger.clone(), &window_clone);
        input.set_text("");
        window_clone.show_all();
        dialog.hide();
    });
}

pub fn render(builder: &gtk::Builder, window: &gtk::Window, logger: Arc<Logger>) {
    run_dialog(&builder);

    boton_cancel_dialog(&builder);
    boton_confimar_dialog(&builder, &window, logger);
}

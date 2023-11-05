mod branch_selector;
mod clone_dialog;
mod log_list;
mod log_seleccionado;
mod new_branch_dialog;
mod new_commit_dialog;
mod staging_area;

use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

use crate::tipos_de_dato::comandos::commit::Commit;
use crate::tipos_de_dato::logger::Logger;
use gtk::prelude::*;
use gtk::{self};

pub fn ejecutar(logger: Arc<Logger>) {
    if gtk::init().is_err() {
        println!("Failed to initialize GTK.");
        return;
    }

    let glade_src = include_str!("glade1.glade");
    let builder = gtk::Builder::from_string(glade_src);
    let window: gtk::Window = builder.object("home").unwrap();
    window.set_position(gtk::WindowPosition::Center);

    if !PathBuf::from(".gir").is_dir() && !clone_dialog::render(&builder, logger.clone()) {
        return;
    }

    let branch_actual = Commit::obtener_branch_actual().unwrap();

    new_branch_dialog::render(&builder, &window, logger.clone());
    branch_selector::render(&builder, &window, logger.clone());
    log_list::render(&builder, branch_actual);
    log_seleccionado::render(&builder, None);
    staging_area::render(&builder, &window, logger.clone());
    new_commit_dialog::render(&builder, &window, logger.clone());

    window.show_all();

    gtk::main();
}

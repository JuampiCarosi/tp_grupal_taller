mod branch_selector;
mod clone_dialog;
mod error_dialog;
mod log_list;
mod log_seleccionado;
mod new_branch_dialog;
mod new_commit_dialog;
mod pull_button;
mod push_button;
mod refresh;
mod staging_area;


use std::net::TcpStream;
use std::path::PathBuf;
use std::sync::Arc;


use crate::tipos_de_dato::comandos::commit::Commit;
use crate::tipos_de_dato::comunicacion::{Comunicacion};
use crate::tipos_de_dato::logger::Logger;
use gtk::prelude::*;
use gtk::{self};

fn hidratar_componentes(
    builder: &gtk::Builder,
    window: &gtk::Window,
    logger: Arc<Logger>,
    branch_actual: String,
    comunicacion: Arc<Comunicacion<TcpStream>>,
) {
    new_branch_dialog::render(&builder, &window, logger.clone());
    branch_selector::render(&builder, &window, logger.clone());
    log_list::render(&builder, branch_actual.clone());
    log_seleccionado::render(&builder, None);
    staging_area::render(&builder, &window, logger.clone());
    new_commit_dialog::render(&builder, &window, logger.clone());
    push_button::render(&builder, &window, comunicacion.clone());
    error_dialog::setup(&builder);
    pull_button::render(
        &builder,
        &window,
        comunicacion.clone(),
        logger.clone(),
        branch_actual.clone(),
    );
    refresh::render(
        &builder,
        &window,
        logger.clone(),
        branch_actual.clone(),
        comunicacion.clone(),
    );
}

pub fn ejecutar(logger: Arc<Logger>, comunicacion: Arc<Comunicacion<TcpStream>>) {
    if gtk::init().is_err() {
        println!("Failed to initialize GTK.");
        return;
    }

    let glade_src = include_str!("glade1.glade");
    let builder = gtk::Builder::from_string(glade_src);
    let window: gtk::Window = builder.object("home").unwrap();
    window.set_position(gtk::WindowPosition::Center);

    if !PathBuf::from(".gir").is_dir()
        && !clone_dialog::render(&builder, logger.clone(), comunicacion.clone())
    {
        return;
    }

    let branch_actual = Commit::obtener_branch_actual().unwrap();

    hidratar_componentes(
        &builder,
        &window,
        logger.clone(),
        branch_actual,
        comunicacion,
    );

    window.show_all();

    window.connect_delete_event(|_, _| {
        gtk::main_quit();
        gtk::glib::Propagation::Proceed
    });

    gtk::main();
}

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

use std::path::PathBuf;
use std::sync::Arc;

use crate::tipos_de_dato::comandos::commit::Commit;
use crate::tipos_de_dato::logger::Logger;
use gtk::{self, Settings, StyleContext};
use gtk::{gdk, prelude::*};

fn hidratar_componentes(
    builder: &gtk::Builder,
    window: &gtk::Window,
    logger: Arc<Logger>,
    branch_actual: String,
) {
    estilos();
    new_branch_dialog::render(builder, window, logger.clone());
    branch_selector::render(builder, window, logger.clone());
    log_list::render(builder, branch_actual.clone());
    log_seleccionado::render(builder, None);
    staging_area::render(builder, window, logger.clone());
    new_commit_dialog::render(builder, window, logger.clone());
    push_button::render(builder, window, logger.clone());
    error_dialog::setup(builder);
    pull_button::render(builder, window, logger.clone(), branch_actual.clone());
    refresh::render(builder, window, logger.clone(), branch_actual.clone());
}

pub fn estilos() {
    let css_provider = gtk::CssProvider::new();
    css_provider
        .load_from_data(include_str!("estilos.css").as_bytes())
        .unwrap();

    let screen = gdk::Screen::default().unwrap();
    StyleContext::add_provider_for_screen(
        &screen,
        &css_provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}

pub fn ejecutar(logger: Arc<Logger>) {
    if gtk::init().is_err() {
        println!("Failed to initialize GTK.");
        return;
    }

    std::env::set_var("GSK_RENDERER", "cairo");

    let glade_src = include_str!("glade1.glade");
    let builder = gtk::Builder::from_string(glade_src);
    let window: gtk::Window = builder.object("home-v2").unwrap();
    window.set_position(gtk::WindowPosition::Center);
    window.set_default_size(800, 600);

    if !PathBuf::from(".gir").is_dir() && !clone_dialog::render(&builder, logger.clone()) {
        return;
    }

    let branch_actual = Commit::obtener_branch_actual().unwrap();

    hidratar_componentes(&builder, &window, logger.clone(), branch_actual);

    window.show_all();

    window.connect_delete_event(|_, _| {
        gtk::main_quit();
        gtk::glib::Propagation::Proceed
    });

    gtk::main();
}

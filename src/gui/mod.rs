mod branch_selector;
mod clone_dialog;
mod conflicts_modal;
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

use crate::tipos_de_dato::logger::Logger;
use crate::utils::ramas;
use gtk::{self, StyleContext};
use gtk::{gdk, prelude::*};

fn hidratar_componentes(
    builder: &gtk::Builder,
    window: &gtk::Window,
    logger: Arc<Logger>,
    branch_actual: &str,
) {
    let screen = gdk::Screen::default().unwrap();
    estilos(screen);
    new_branch_dialog::render(builder, window, logger.clone());
    branch_selector::render(builder, window, logger.clone());
    log_list::render(builder, branch_actual, logger.clone());
    log_seleccionado::render(builder, None);
    staging_area::render(builder, window, logger.clone());
    new_commit_dialog::render(builder, window, logger.clone());
    push_button::render(builder, window, logger.clone());
    error_dialog::setup(builder);
    pull_button::render(builder, window, logger.clone(), branch_actual.to_string());
    conflicts_modal::render(builder, logger.clone());
    refresh::render(builder, window, logger.clone(), branch_actual.to_string());
}

pub fn estilos(screen: gdk::Screen) {
    let css_provider = gtk::CssProvider::new();
    css_provider
        .load_from_data(include_str!("estilos.css").as_bytes())
        .unwrap();

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

    let branch_actual = ramas::obtener_rama_actual().unwrap();

    hidratar_componentes(&builder, &window, logger.clone(), &branch_actual);

    window.show_all();

    window.connect_delete_event(|_, _| {
        gtk::main_quit();
        gtk::glib::Propagation::Proceed
    });

    gtk::main();
}

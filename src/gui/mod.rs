mod branch_selector;
mod log_list;
mod log_seleccionado;
mod new_branch_dialog;
mod new_commit_dialog;
mod staging_area;

use std::rc::Rc;

use crate::tipos_de_dato::comandos::commit::Commit;
use crate::tipos_de_dato::logger::{self, Logger};
use gtk::prelude::*;
use gtk::{self};

pub fn ejecutar(logger: Rc<Logger>) {
    if gtk::init().is_err() {
        println!("Failed to initialize GTK.");
        return;
    }

    let glade_src = include_str!("glade1.glade");
    let builder = gtk::Builder::from_string(glade_src);
    let branch_actual = Commit::obtener_branch_actual().unwrap();

    let window: gtk::Window = builder.object("home").unwrap();
    window.set_position(gtk::WindowPosition::Center);

    new_branch_dialog::render(&builder, &window);
    branch_selector::render(&builder, &window);
    log_list::render(&builder, branch_actual);
    log_seleccionado::render(&builder, None);
    staging_area::render(&builder, logger.clone(), &window);
    new_commit_dialog::render(&builder, &window, logger.clone());

    window.show_all();

    window.connect_delete_event(|_, _| {
        gtk::main_quit();
        gtk::glib::Propagation::Proceed
    });

    gtk::main();
}

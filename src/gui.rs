use std::path::PathBuf;
use std::rc::Rc;

use gtk;
use gtk::prelude::*;

use crate::tipos_de_dato::comandos::branch::{self, Branch};
use crate::tipos_de_dato::logger::Logger;

fn branch_dialog(builder: gtk::Builder) {
    let branch_button: gtk::Button = builder.object("branch-button").unwrap();
    let dialog: gtk::MessageDialog = builder.object("branch").unwrap();

    dialog.set_position(gtk::WindowPosition::Center);

    branch_button.connect_clicked(move |_| {
        dialog.run();
        dialog.hide();
    });

    let confirm: gtk::Button = builder.object("confirm-branch").unwrap();
    let cancel: gtk::Button = builder.object("cancel-branch").unwrap();

    let builder_clone = builder.clone();
    confirm.connect_clicked(move |_| {
        let dialog: gtk::MessageDialog = builder_clone.object("branch").unwrap();
        let input: gtk::Entry = builder_clone.object("branch-input").unwrap();
        println!("branch name: {}", input.text());
        dialog.hide();
    });

    cancel.connect_clicked(move |_| {
        let dialog: gtk::MessageDialog = builder.object("branch").unwrap();
        let input: gtk::Entry = builder.object("branch-input").unwrap();
        input.set_text("");
        dialog.hide();
    });
}

fn select_branch(builder: gtk::Builder) {
    let select: gtk::ComboBoxText = builder.object("select-branch").unwrap();

    let logger = Rc::new(Logger::new(PathBuf::from("tmp/branch_gui")).unwrap());

    let binding = Branch::mostrar_ramas().unwrap();
    let branches = binding.split("\n");

    branches.for_each(|branch| {
        if branch == "" {
            return;
        }
        select.append_text(branch);
    });

    select.set_active(Some(0 as u32));

    select.connect_changed(|select| {
        let active = select.active_text().unwrap();
        println!("active: {}", active);
    });
}

pub fn ejecutar() {
    if gtk::init().is_err() {
        println!("Failed to initialize GTK.");
        return;
    }

    let glade_src = include_str!("glade1.glade");
    let builder = gtk::Builder::from_string(glade_src);

    let window: gtk::Window = builder.object("home").unwrap();
    window.set_position(gtk::WindowPosition::Center);

    branch_dialog(builder.clone());
    select_branch(builder.clone());

    window.show_all();

    gtk::main();
}

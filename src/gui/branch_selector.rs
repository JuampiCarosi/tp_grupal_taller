use std::{path::PathBuf, rc::Rc};

use gtk::prelude::*;

use crate::tipos_de_dato::{
    comandos::{branch::Branch, checkout::Checkout, commit::Commit},
    logger::Logger,
};

use super::{log_list, log_seleccionado};

pub fn render(builder: &gtk::Builder, window: &gtk::Window) {
    let select: gtk::ComboBoxText = builder.object("select-branch").unwrap();
    let branch_actual = Commit::obtener_branch_actual().unwrap();
    select.remove_all();

    let binding = Branch::mostrar_ramas().unwrap();
    let branches = binding.split("\n");

    let mut i = 0;
    branches.for_each(|branch| {
        if branch == "" {
            return;
        }
        select.append_text(branch);
        if branch == branch_actual {
            select.set_active(Some(i));
        }
        i += 1;
    });

    let builder_clone = builder.clone();
    let window_clone = window.clone();
    select.connect_changed(move |a| {
        let logger = Rc::new(Logger::new(PathBuf::from("log.txt")).unwrap());

        let active = match a.active_text() {
            Some(text) => text,
            None => return,
        };

        log_list::render(&builder_clone, active.to_string());
        log_seleccionado::render(&builder_clone, None);
        let _ = Checkout::from(vec![active.to_string()], logger)
            .unwrap()
            .ejecutar();
        window_clone.show_all();
    });
}

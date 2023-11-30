use std::sync::Arc;

use gtk::prelude::*;

use crate::tipos_de_dato::{
    comando::Ejecutar,
    comandos::{branch::Branch, checkout::Checkout, commit::Commit},
    logger::Logger,
};

use super::{error_dialog, log_list, log_seleccionado};

pub fn render(builder: &gtk::Builder, window: &gtk::Window, logger: Arc<Logger>) {
    let select: gtk::ComboBoxText = builder.object("select-branch").unwrap();
    let branch_actual = Commit::obtener_branch_actual().unwrap();
    select.remove_all();

    let branches = match Branch::obtener_ramas() {
        Ok(branches) => branches,
        Err(err) => {
            error_dialog::mostrar_error(&err);
            return;
        }
    };

    let mut i = 0;
    branches.iter().for_each(|branch| {
        if branch.is_empty() {
            return;
        }
        select.append_text(branch);
        if *branch == branch_actual {
            select.set_active(Some(i));
        }
        i += 1;
    });

    let builder_clone = builder.clone();
    let window_clone = window.clone();
    select.connect_changed(move |a| {
        let active = match a.active_text() {
            Some(text) => text,
            None => return,
        };

        log_list::render(&builder_clone, active.as_str());
        log_seleccionado::render(&builder_clone, None);

        let mut checkout = Checkout::from(vec![active.to_string()], logger.clone()).unwrap();

        match checkout.ejecutar() {
            Ok(_) => {}
            Err(err) => {
                error_dialog::mostrar_error(&err);
            }
        }

        window_clone.show_all();
    });
}

use std::sync::Arc;

use gtk::prelude::*;

use super::{comando_gui::ComandoGui, conflicts_modal, error_dialog};
use crate::tipos_de_dato::{
    comandos::{branch::Branch, merge::Merge, rebase::Rebase},
    logger::Logger,
};

pub enum AccionBranchDialog {
    Merge,
    Rebase,
}

fn obtener_ramas_disponibles() -> Vec<String> {
    let todas = match Branch::mostrar_ramas() {
        Ok(ramas) => ramas,
        Err(err) => {
            error_dialog::mostrar_error(&err);
            return vec![];
        }
    };
    return todas
        .lines()
        .filter_map(|rama| {
            if rama.starts_with("*") {
                None
            } else {
                Some(rama.trim().to_string())
            }
        })
        .collect();
}

pub fn render(builder: &gtk::Builder, logger: Arc<Logger>, accion: AccionBranchDialog) {
    let dialog = builder.object::<gtk::Dialog>("branch-dialog").unwrap();

    let confirmar = builder.object::<gtk::Button>("confirm-branc").unwrap();

    let combobox = builder
        .object::<gtk::ComboBoxText>("branch-combo-box")
        .unwrap();

    dialog.set_position(gtk::WindowPosition::Center);

    let ramas = obtener_ramas_disponibles();
    if ramas.is_empty() {
        return;
    }

    for rama in ramas {
        combobox.append_text(&rama);
    }

    let dialog_clone = dialog.clone();
    let builder_clone = builder.clone();
    confirmar.connect_clicked(move |_| {
        let activo = match combobox.active_text() {
            Some(activo) => activo,
            None => return,
        };
        let mut args = vec![activo.to_string()];
        match accion {
            AccionBranchDialog::Merge => Merge::from(&mut args, logger.clone()).ejecutar_gui(),
            AccionBranchDialog::Rebase => Rebase::from(args, logger.clone()).ejecutar_gui(),
        };

        conflicts_modal::boton_conflictos(&builder_clone, logger.clone());

        dialog_clone.hide();
    });

    dialog.run();
}

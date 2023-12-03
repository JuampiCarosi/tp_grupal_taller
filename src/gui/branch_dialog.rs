use std::sync::Arc;

use gtk::prelude::*;

use super::comando_gui::ComandoGui;
use crate::tipos_de_dato::{
    comando::Ejecutar,
    // comandos::{merge::Merge, rebase::Rebase},
    logger::Logger,
};

pub enum AccionBranchDialog {
    Merge,
    Rebase,
}

pub fn render<T>(builder: gtk::Builder, logger: Arc<Logger>, accion: AccionBranchDialog) {
    let dialog = builder.object::<gtk::Dialog>("branch-dialog").unwrap();
    let confirmar = builder.object::<gtk::Button>("confirm-branch").unwrap();
    let combobox = builder
        .object::<gtk::ComboBoxText>("branch-combo-box")
        .unwrap();

    dialog.set_position(gtk::WindowPosition::Center);

    confirmar.connect_clicked(move |_| {
        let activo = combobox.active_text().unwrap();
        let args = vec![activo.to_string()];
        // match accion {
        // AccionBranchDialog::Merge => Merge::from(&mut args, logger.clone()).ejecutar_gui(),
        // AccionBranchDialog::Rebase => Rebase::from(&mut args, logger.clone()).ejecutar_gui(),
        // }

        dialog.close();
    });

    // dialog.run();
}

use gtk::glib::Propagation;
use gtk::prelude::*;
use gtk::{self};
use std::sync::Arc;

use crate::gui::comando_gui::ComandoGui;
use crate::tipos_de_dato::comandos::commit::Commit;
use crate::tipos_de_dato::comandos::push::Push;
use crate::tipos_de_dato::comandos::set_upstream::SetUpstream;
use crate::tipos_de_dato::config::Config;
use crate::tipos_de_dato::logger::Logger;

use super::info_dialog;

fn setear_remoto(builder: &gtk::Builder, logger: Arc<Logger>) {
    let rama_actual = Commit::obtener_branch_actual().unwrap();

    let remoto = builder
        .object::<gtk::Entry>("remote-u")
        .unwrap()
        .text()
        .to_string();
    let rama_remota = builder
        .object::<gtk::Entry>("branch-u")
        .unwrap()
        .text()
        .to_string();

    SetUpstream::new(remoto, rama_remota, rama_actual, logger.clone()).ejecutar_gui();
}

fn upstream_modal(builder: &gtk::Builder, logger: Arc<Logger>) {
    let config = Config::leer_config().unwrap();
    let rama_actual = Commit::obtener_branch_actual().unwrap();
    let hay_upstream = config.hay_upstream(&rama_actual);

    if hay_upstream {
        return;
    }

    let dialog = builder
        .object::<gtk::MessageDialog>("set-upstream-dialog")
        .unwrap();

    let confirmar = builder
        .object::<gtk::Button>("confirm-set-upstream")
        .unwrap();
    let cancelar = builder
        .object::<gtk::Button>("cancel-set-upstream")
        .unwrap();

    let dialog_clone = dialog.clone();
    let builder_clone = builder.clone();
    let confirmar_id = confirmar.connect_clicked(move |_| {
        setear_remoto(&builder_clone, logger.clone());
        dialog_clone.hide();
    });

    let dialog_clone = dialog.clone();
    let cancelar_id = cancelar.connect_clicked(move |_| {
        dialog_clone.hide();
    });

    let destroy_id = dialog.connect_destroy_event(|dialog, _| {
        dialog.hide();
        Propagation::Stop
    });

    dialog.run();
    dialog.disconnect(destroy_id);
    dialog.disconnect(confirmar_id);
    dialog.disconnect(cancelar_id);
}

pub fn render(builder: &gtk::Builder, logger: Arc<Logger>) {
    let push_button = builder.object::<gtk::Button>("push-button").unwrap();

    let builder_clone = builder.clone();
    push_button.connect_clicked(move |_| {
        upstream_modal(&builder_clone, logger.clone());

        let resultado = Push::new(&mut Vec::new(), logger.clone()).ejecutar_gui();

        if let Some(resultado) = resultado {
            info_dialog::mostrar_mensaje(&resultado, "");
        }
    });
}

use std::sync::Arc;

use gtk::prelude::*;

use crate::tipos_de_dato::{comandos::commit::Commit, logger::Logger};

use super::hidratar_componentes;

pub fn render(builder: &gtk::Builder, window: &gtk::Window, logger: Arc<Logger>) {
    let icon = builder.object::<gtk::Button>("refresh-button").unwrap();

    let builder = builder.clone();
    let window = window.clone();
    icon.connect_clicked(move |_| {
        let branch_actual = Commit::obtener_branch_actual().unwrap();
        hidratar_componentes(&builder, &window, logger.clone(), &branch_actual);
    });
}

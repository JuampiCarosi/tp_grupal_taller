use std::sync::Arc;

use gtk::prelude::*;

use crate::tipos_de_dato::{comandos::commit::Commit, logger::Logger};

use super::hidratar_componentes;

pub fn render(builder: &gtk::Builder, logger: Arc<Logger>) {
    let icon = builder.object::<gtk::Button>("refresh-button").unwrap();

    let builder = builder.clone();
    icon.connect_clicked(move |_| {
        let branch_actual = Commit::obtener_branch_actual().unwrap();
        hidratar_componentes(&builder, logger.clone(), &branch_actual);
    });
    icon.show_all();
}

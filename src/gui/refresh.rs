use std::sync::Arc;

use gtk::prelude::*;

use crate::tipos_de_dato::logger::Logger;

use super::hidratar_componentes;

pub fn render(
    builder: &gtk::Builder,
    window: &gtk::Window,
    logger: Arc<Logger>,
    branch_actual: String,
) {
    let icon = builder.object::<gtk::Button>("refresh-button").unwrap();

    let builder = builder.clone();
    let window = window.clone();
    icon.connect_clicked(move |_| {
        hidratar_componentes(&builder, &window, logger.clone(), branch_actual.clone());
    });
}

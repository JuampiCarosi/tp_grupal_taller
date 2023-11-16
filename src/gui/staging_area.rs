use std::sync::Arc;

use crate::{
    tipos_de_dato::{
        comandos::{add::Add, status::Status},
        logger::Logger,
    },
    utils::index::{leer_index, ObjetoIndex},
};
use gtk::prelude::*;

fn crear_label_rojo(string: &str) -> gtk::EventBox {
    let event_box = gtk::EventBox::new();
    let label = gtk::Label::new(Some(string));
    label.set_xalign(0.0);
    event_box.add(&label);

    let css_provider = gtk::CssProvider::new();
    css_provider
        .load_from_data(
            "
             .verde  {
                font-size: 12px;
                font-family: monospace;
                color: rgb(240, 68, 56);
            }

        "
            .as_bytes(),
        )
        .unwrap();

    let context = event_box.style_context();

    gtk::StyleContext::add_provider(
        &context,
        &css_provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    event_box.style_context().add_class("verde");

    event_box
}

fn crear_label_verde(string: &str) -> gtk::EventBox {
    let event_box = gtk::EventBox::new();
    let label = gtk::Label::new(Some(string));
    label.set_xalign(0.0);
    event_box.add(&label);

    let css_provider = gtk::CssProvider::new();
    css_provider
        .load_from_data(
            "
             .rojo  {
                font-size: 12px;
                font-family: monospace;
                color: rgb(3, 152, 85);
            }

        "
            .as_bytes(),
        )
        .unwrap();

    let context = event_box.style_context();

    gtk::StyleContext::add_provider(
        &context,
        &css_provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    event_box.style_context().add_class("rojo");

    event_box
}

fn extraer_path(objeto: &ObjetoIndex) -> String {
    objeto
        .objeto
        .obtener_path()
        .as_os_str()
        .to_str()
        .unwrap()
        .to_string()
}

fn escribir_archivos_index(builder: &gtk::Builder, logger: Arc<Logger>) {
    let index = leer_index(logger.clone()).unwrap();

    let label = crear_label_verde("Archivos en staging:");

    let list_box: gtk::Box = builder.object("staging").unwrap();
    list_box.add(&label);

    for objeto_index in index {
        let path = extraer_path(&objeto_index);
        let label = crear_label_verde(&format!("    {}", path));

        let list_box: gtk::Box = builder.object("staging").unwrap();
        list_box.add(&label);
        logger.log("Gui: Agregando archivo a staging".to_string());
    }
}

fn estilar_lista_archivos(builder: &gtk::Builder) {
    // let window: gtk::ScrolledWindow = builder.object("scrolled-window").unwrap();

    // window.style_context().add_class("contenedor-commits");
}

fn escribir_archivos_modificados(
    builder: &gtk::Builder,
    logger: Arc<Logger>,
    window: &gtk::Window,
) {
    let status = Status::from(logger.clone()).unwrap();
    let lineas = status.obtener_trackeados().unwrap();

    let container: gtk::Box = builder.object("staging").unwrap();

    let label = crear_label_rojo("\nArchivos con cambios:");
    container.pack_start(&label, false, false, 0);

    for linea in lineas {
        let split = linea.split(": ").collect::<Vec<&str>>();
        let nombre = split[1];

        let label = crear_label_rojo(&format!("+ {nombre}",));

        let logger_callback = logger.clone();
        let nombre_callback = nombre.to_string();
        let builder_callback = builder.clone();
        let window_callback = window.clone();
        label.connect_button_press_event(move |_, _| {
            logger_callback.log("Gui: Agregando archivo a staging".to_string());
            let mut add =
                Add::from(vec![nombre_callback.clone()], logger_callback.clone()).unwrap();
            add.ejecutar().unwrap();

            render(&builder_callback, &window_callback, logger_callback.clone());

            gtk::glib::Propagation::Proceed
        });

        container.pack_start(&label, false, true, 0);
    }
}

fn escribir_archivos_untrackeados(
    builder: &gtk::Builder,
    logger: Arc<Logger>,
    window: &gtk::Window,
) {
    let status = Status::from(logger.clone()).unwrap();
    let lineas = status.obtener_untrackeados().unwrap();

    let container: gtk::Box = builder.object("staging").unwrap();

    let label = crear_label_rojo("\nArchivos sin trackear:");
    container.pack_start(&label, false, false, 0);

    for linea in lineas {
        let label = crear_label_rojo(&format!("+ {linea}",));

        let logger_callback = logger.clone();
        let nombre_callback = linea.to_string();
        let builder_callback = builder.clone();
        let window_callback = window.clone();
        label.connect_button_press_event(move |_, _| {
            logger_callback.log("Gui: Agregando archivo a staging".to_string());
            let mut add =
                Add::from(vec![nombre_callback.clone()], logger_callback.clone()).unwrap();
            add.ejecutar().unwrap();

            render(&builder_callback, &window_callback, logger_callback.clone());

            gtk::glib::Propagation::Proceed
        });

        container.pack_start(&label, false, true, 0);
    }
}

fn limpiar_archivos(builder: &gtk::Builder) {
    let container: gtk::Box = builder.object("staging").unwrap();
    container.children().iter().for_each(|child| {
        container.remove(child);
    });
}

pub fn render(builder: &gtk::Builder, window: &gtk::Window, logger: Arc<Logger>) {
    logger.log("Gui: Renderizando staging area".to_string());
    limpiar_archivos(builder);
    estilar_lista_archivos(builder);
    escribir_archivos_index(builder, logger.clone());
    escribir_archivos_modificados(builder, logger.clone(), window);
    escribir_archivos_untrackeados(builder, logger, window);
    window.show_all();
}

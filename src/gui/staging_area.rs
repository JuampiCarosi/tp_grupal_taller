use std::rc::Rc;

use crate::{
    tipos_de_dato::{
        comandos::{add::Add, status::Status},
        logger::{self, Logger},
    },
    utilidades_index::{leer_index, ObjetoIndex},
};
use gtk::{glib::collections::list, prelude::*};

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

fn escribir_archivos_index(builder: &gtk::Builder, logger: Rc<Logger>) {
    let index = leer_index().unwrap();

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
    let window: gtk::ScrolledWindow = builder.object("scrolled-window").unwrap();

    let css_provider = gtk::CssProvider::new();
    css_provider
        .load_from_data(
            "
           scrolledwindow  {
              font-size: 12px;
              font-family: monospace;
              color: #00FF00;

              border: 0px solid #000000;
          }

      "
            .as_bytes(),
        )
        .unwrap();

    let context = window.style_context();
    gtk::StyleContext::add_provider(
        &context,
        &css_provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    window.style_context().add_class("contenedor-commits");
}

fn escribir_archivos_modificados(builder: &gtk::Builder, logger: Rc<Logger>, window: &gtk::Window) {
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

            limpiar_archivos(&builder_callback);
            escribir_archivos_index(&builder_callback, logger_callback.clone());
            escribir_archivos_modificados(
                &builder_callback,
                logger_callback.clone(),
                &window_callback,
            );
            window_callback.show_all();

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

pub fn render(builder: &gtk::Builder, logger: Rc<Logger>, window: &gtk::Window) {
    logger.log("Gui: Renderizando staging area".to_string());
    estilar_lista_archivos(&builder);
    escribir_archivos_index(&builder, logger.clone());
    escribir_archivos_modificados(&builder, logger, window);
}

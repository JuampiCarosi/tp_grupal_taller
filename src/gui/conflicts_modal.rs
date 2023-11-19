use std::sync::Arc;

use gtk::{gdk, prelude::*, Button};

use crate::{
    tipos_de_dato::{comandos::merge::Merge, logger::Logger, objeto::Objeto},
    utils::{index::leer_index, io::leer_a_string},
};

use super::estilos;

fn conflicts_button(builder: &gtk::Builder, logger: Arc<Logger>) {
    let boton: Button = builder.object("conflicts-button").unwrap();
    if !Merge::hay_archivos_sin_mergear(logger.clone()).unwrap() {
        boton.set_sensitive(false);
    }

    let builder = builder.clone();
    boton.connect_clicked(move |_| {
        modal(&builder, logger.clone());
    });
}

fn linea_head(linea: &str) -> String {
    "<span bgcolor=\"#5eead4\" >".to_string() + &linea.replace("<", "&lt;") + "</span>\n"
}

fn linea_incoming(linea: &str) -> String {
    "<span bgcolor=\"#67e8f9\" >".to_string() + &linea + "</span>\n"
}

fn linea_contenido_head(linea: &str) -> String {
    "<span bgcolor=\"#99f6e4\" >".to_string() + &linea + "</span>\n"
}

fn linea_contenido_incoming(linea: &str) -> String {
    "<span bgcolor=\"#a5f3fc\" >".to_string() + &linea + "</span>\n"
}

fn resaltar_conflictos(buffer: &gtk::TextBuffer) {
    let texto = buffer
        .text(&buffer.start_iter(), &buffer.end_iter(), false)
        .unwrap();
    let lineas = texto.split('\n').collect::<Vec<&str>>();
    buffer.set_text("");
    let mut i = 0;
    while i < lineas.len() {
        if lineas[i].starts_with("<<<<<<<") {
            buffer.insert_markup(&mut buffer.end_iter(), &linea_head(lineas[i]));
            i += 1;
            while i < lineas.len() && !lineas[i].starts_with("=======") {
                buffer.insert_markup(&mut buffer.end_iter(), &linea_contenido_head(lineas[i]));

                i += 1;
            }
            if i < lineas.len() {
                buffer.insert_markup(&mut buffer.end_iter(), &(lineas[i].to_string() + "\n"));

                i += 1;
                while i < lineas.len() && !lineas[i].starts_with(">>>>>>>") {
                    buffer.insert_markup(
                        &mut buffer.end_iter(),
                        &linea_contenido_incoming(lineas[i]),
                    );

                    i += 1;
                }
                if i < lineas.len() {
                    buffer.insert_markup(&mut buffer.end_iter(), &linea_incoming(lineas[i]));

                    i += 1;
                }
            }
        } else {
            buffer.insert_markup(&mut buffer.end_iter(), &(lineas[i].to_string() + "\n"));
            i += 1;
        }
    }
}

fn crear_text_area_de_objeto(objeto: &Objeto) -> gtk::TextView {
    let text = gtk::TextView::new();
    text.set_left_margin(5);
    text.set_right_margin(5);
    text.set_top_margin(5);
    text.set_bottom_margin(5);
    text.set_monospace(true);
    let contenido = leer_a_string(objeto.obtener_path()).unwrap();
    let buffer = text.buffer().unwrap();
    buffer.set_text(&contenido);
    resaltar_conflictos(&buffer);

    text
}

fn contenedor(objeto: &Objeto) -> gtk::Box {
    let contenedor = gtk::Box::new(gtk::Orientation::Vertical, 0);
    contenedor.set_margin_start(5);
    contenedor.set_margin_end(5);
    contenedor.set_margin_top(5);
    contenedor.set_margin_bottom(5);
    let texto_1 = crear_text_area_de_objeto(objeto);
    let texto_2 = crear_text_area_de_objeto(objeto);

    texto_1.style_context().add_class("text-red");

    contenedor.add(&texto_1);
    contenedor.add(&texto_2);
    contenedor
}

fn crear_notebook(builder: &gtk::Builder, logger: Arc<Logger>) {
    let index = leer_index(logger).unwrap();
    let sin_mergear: Vec<_> = index.iter().filter(|objeto| objeto.merge).collect();
    let notebook: gtk::Notebook = builder.object("conflicts-notebook").unwrap();
    for objeto_viejo in notebook.children() {
        notebook.remove(&objeto_viejo);
    }
    for objeto in sin_mergear {
        let text_area = contenedor(&objeto.objeto);
        let label = gtk::Label::new(Some(
            &objeto.objeto.obtener_path().to_string_lossy().to_string(),
        ));
        notebook.append_page(&text_area, Some(&label));
    }
}

fn modal(builder: &gtk::Builder, logger: Arc<Logger>) {
    let modal: gtk::Window = builder.object("conflicts-window").unwrap();
    modal.set_position(gtk::WindowPosition::Center);
    crear_notebook(builder, logger.clone());

    let screen = gdk::Screen::default().unwrap();
    estilos(screen);

    modal.connect_delete_event(|modal, _| {
        modal.hide();
        gtk::glib::Propagation::Stop
    });
    modal.set_position(gtk::WindowPosition::Center);
    modal.show_all();
}

pub fn render(builder: &gtk::Builder, window: &gtk::Window, logger: Arc<Logger>) {
    conflicts_button(builder, logger.clone());
}

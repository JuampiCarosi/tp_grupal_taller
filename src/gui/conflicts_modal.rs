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

fn resaltar_linea(buffer: &gtk::TextBuffer, numero_linea: i32, tag: &str) {
    let start_iter = buffer.iter_at_line(numero_linea);
    let end_iter = buffer.iter_at_line(numero_linea + 1);
    buffer.apply_tag_by_name(tag, &start_iter, &end_iter);
}

fn limpiar_resaltado_linea(buffer: &gtk::TextBuffer, numero_linea: i32) {
    let start_iter = buffer.iter_at_line(numero_linea);
    let end_iter = buffer.iter_at_line(numero_linea + 1);
    buffer.remove_all_tags(&start_iter, &end_iter);
}

fn crear_tags(buffer: &gtk::TextBuffer) {
    let head_titulo = gtk::TextTag::new(Some("head_titulo"));
    head_titulo.set_paragraph_background(Some("#5eead4"));
    let head_contenido = gtk::TextTag::new(Some("head_contenido"));
    head_contenido.set_paragraph_background(Some("#99f6e4"));
    let incoming_titulo = gtk::TextTag::new(Some("incoming_titulo"));
    incoming_titulo.set_paragraph_background(Some("#67e8f9"));
    let incoming_contenido = gtk::TextTag::new(Some("incoming_contenido"));
    incoming_contenido.set_paragraph_background(Some("#a5f3fc"));
    let table = buffer.tag_table().unwrap();
    table.add(&head_titulo);
    table.add(&head_contenido);
    table.add(&incoming_titulo);
    table.add(&incoming_contenido);
}

#[derive(PartialEq)]
enum Estado {
    Head,
    Incoming,
    None,
}

fn resaltar_conflictos(buffer: &gtk::TextBuffer) {
    let texto = buffer
        .text(&buffer.start_iter(), &buffer.end_iter(), false)
        .unwrap();
    let lineas = texto.split('\n').collect::<Vec<&str>>();
    let mut estado = Estado::None;
    for (i, linea) in lineas.iter().enumerate() {
        match *linea {
            l if l.starts_with("<<<<<<<") => {
                resaltar_linea(buffer, i as i32, "head_titulo");
                estado = Estado::Head;
                continue;
            }
            l if l.starts_with(">>>>>>>") => {
                resaltar_linea(buffer, i as i32, "incoming_titulo");
                estado = Estado::None;
                continue;
            }
            "=======" => {
                estado = Estado::Incoming;
                continue;
            }
            _ => {}
        }
        match estado {
            Estado::Head => {
                resaltar_linea(buffer, i as i32, "head_contenido");
            }
            Estado::Incoming => {
                resaltar_linea(buffer, i as i32, "incoming_contenido");
            }
            Estado::None => {}
        }
    }
}

fn crear_text_area_de_objeto(objeto: &Objeto) -> gtk::ScrolledWindow {
    let scrollable_window = gtk::ScrolledWindow::new(gtk::Adjustment::NONE, gtk::Adjustment::NONE);
    scrollable_window.set_shadow_type(gtk::ShadowType::None);
    scrollable_window.set_height_request(400);
    scrollable_window.set_width_request(600);
    let viewport = gtk::Viewport::new(gtk::Adjustment::NONE, gtk::Adjustment::NONE);
    let text = gtk::TextView::new();

    scrollable_window.add(&viewport);
    viewport.add(&text);

    text.set_left_margin(5);
    text.set_right_margin(5);
    text.set_top_margin(5);
    text.set_bottom_margin(5);
    text.set_monospace(true);
    let contenido = leer_a_string(objeto.obtener_path()).unwrap();
    let buffer = text.buffer().unwrap();
    buffer.set_text(&contenido);
    crear_tags(&buffer);
    resaltar_conflictos(&buffer);
    buffer.connect_changed(|buffer| {
        resaltar_conflictos(buffer);
    });

    scrollable_window
}

fn crear_notebook(builder: &gtk::Builder, logger: Arc<Logger>) {
    let index = leer_index(logger).unwrap();
    let sin_mergear: Vec<_> = index.iter().filter(|objeto| objeto.merge).collect();
    let notebook: gtk::Notebook = builder.object("conflicts-notebook").unwrap();
    notebook.set_vexpand(true);
    for objeto_viejo in notebook.children() {
        notebook.remove(&objeto_viejo);
    }

    for objeto in sin_mergear {
        let text_area = crear_text_area_de_objeto(&objeto.objeto);
        let label = gtk::Label::new(Some(
            &objeto.objeto.obtener_path().to_string_lossy().to_string(),
        ));
        notebook.append_page(&text_area, Some(&label));
        println!("{}", objeto.objeto.obtener_path().display());
        text_area.show();
    }
}

fn modal(builder: &gtk::Builder, logger: Arc<Logger>) {
    let modal: gtk::Window = builder.object("conflicts-window").unwrap();
    modal.set_position(gtk::WindowPosition::Center);
    crear_notebook(builder, logger.clone());

    modal.connect_delete_event(|modal, _| {
        modal.hide();
        gtk::glib::Propagation::Stop
    });
    modal.set_position(gtk::WindowPosition::Center);
    modal.show();
}

pub fn render(builder: &gtk::Builder, window: &gtk::Window, logger: Arc<Logger>) {
    conflicts_button(builder, logger.clone());
}

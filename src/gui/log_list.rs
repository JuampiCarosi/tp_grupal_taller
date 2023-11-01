use std::path::Path;

use gtk::prelude::*;

use crate::{
    io::leer_a_string, tipos_de_dato::comandos::log::Log,
    utilidades_de_compresion::descomprimir_objeto,
};

use super::log_seleccionado;

fn obtener_listas_de_commits(branch: &String) -> Result<Vec<String>, String> {
    let ruta = format!(".gir/refs/heads/{}", branch);
    let mut ultimo_commit = leer_a_string(Path::new(&ruta))?;

    if ultimo_commit.is_empty() {
        return Ok(Vec::new());
    }
    let mut historial_commits: Vec<String> = Vec::new();
    loop {
        let contenido = descomprimir_objeto(ultimo_commit.clone())?;
        let siguiente_padre = Log::conseguir_padre_desde_contenido_commit(&contenido);
        historial_commits.push(ultimo_commit.clone());
        if siguiente_padre.is_empty() {
            break;
        }
        ultimo_commit = siguiente_padre.to_string();
    }

    Ok(historial_commits)
}

pub fn obtener_mensaje_commit(commit_hash: String) -> String {
    let commit = descomprimir_objeto(commit_hash).unwrap_or("".to_string());

    let mut mensaje = String::new();

    for linea in commit.lines() {
        if !linea.starts_with("commit")
            && !linea.starts_with("parent")
            && !linea.starts_with("author")
            && !linea.starts_with("committer")
            && linea.trim() != ""
        {
            mensaje = linea.trim().to_string();
            break;
        } else {
            mensaje = "Sin mensaje".to_string();
        }
    }

    mensaje
}

fn crear_label(string: &str) -> gtk::EventBox {
    let event_box = gtk::EventBox::new();
    let label = gtk::Label::new(Some(string));
    label.set_xalign(0.0);
    label.set_margin_bottom(4);
    event_box.add(&label);

    let css_provider = gtk::CssProvider::new();
    css_provider
        .load_from_data(
            "
             .custom-label  {
                font-size: 14px;
                font-family: monospace;
                border-bottom: 1px solid #ccc;
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

    event_box.style_context().add_class("custom-label");

    event_box
}

pub fn render(builder: &gtk::Builder, branch: String) {
    let container: gtk::Box = builder.object("log-container").unwrap();
    container.children().iter().for_each(|child| {
        container.remove(child);
    });

    let commits = obtener_listas_de_commits(&branch).unwrap();

    for commit in commits {
        let commit_clone = commit.clone();
        let event_box = crear_label(&obtener_mensaje_commit(commit));

        let builder_clone = builder.clone();
        event_box.connect_button_press_event(move |_, _| {
            log_seleccionado::render(&builder_clone, Some(&commit_clone));
            gtk::glib::Propagation::Stop
        });
        container.add(&event_box);
    }
}

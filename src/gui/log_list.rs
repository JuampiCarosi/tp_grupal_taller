use std::path::Path;

use gtk::prelude::*;

use crate::{
    tipos_de_dato::{comandos::log::Log, objetos::commit::CommitObj},
    utils::{compresion::descomprimir_objeto_gir, io},
};

use super::{error_dialog, log_seleccionado};

fn obtener_listas_de_commits(branch: &str) -> Result<Vec<String>, String> {
    let ruta = format!(".gir/refs/heads/{}", branch);
    let ultimo_commit = io::leer_a_string(Path::new(&ruta))?;

    if ultimo_commit.is_empty() {
        return Ok(Vec::new());
    }

    let commit_obj = CommitObj::from_hash(ultimo_commit)?;
    let historial = Log::obtener_listas_de_commits(commit_obj)?;

    Ok(historial.iter().map(|commit| commit.hash.clone()).collect())
}

pub fn obtener_mensaje_commit(commit_hash: &str) -> Result<String, String> {
    let commit = descomprimir_objeto_gir(commit_hash).unwrap_or("".to_string());

    let mensaje = commit
        .splitn(2, "\n\n")
        .last()
        .ok_or("Error al obtener mensaje del commit")?;

    let primera_linea = mensaje
        .split('\n')
        .next()
        .ok_or("Error al obtener mensaje del commit")?;

    if primera_linea.len() > 35 {
        Ok(format!("{}...", &primera_linea[..35]))
    } else {
        Ok(primera_linea.to_string())
    }
}

fn crear_label(string: &str) -> gtk::EventBox {
    let event_box = gtk::EventBox::new();

    let label = gtk::Label::new(Some(string));
    event_box.add(&label);
    label.set_xalign(0.0);
    label.set_margin_start(6);
    label.set_margin_top(3);
    // label.set_margin_bottom(2);
    event_box.style_context().add_class("commit-label");

    event_box
}

pub fn render(builder: &gtk::Builder, branch: &str) {
    let container: gtk::Box = builder.object("log-container").unwrap();
    container.children().iter().for_each(|child| {
        container.remove(child);
    });

    let commits = match obtener_listas_de_commits(branch) {
        Ok(commits) => commits,
        Err(err) => {
            error_dialog::mostrar_error(&err);
            return;
        }
    };

    for commit in commits {
        let commit_clone = commit.clone();
        let event_box = crear_label(&obtener_mensaje_commit(&commit).unwrap());

        let builder_clone = builder.clone();
        event_box.connect_button_press_event(move |_, _| {
            log_seleccionado::render(&builder_clone, Some(&commit_clone));
            gtk::glib::Propagation::Stop
        });
        container.add(&event_box);
    }
    if !container.children().is_empty() {
        let children = container.children();
        let ultimo = children.last().unwrap();
        ultimo.style_context().add_class("last-commit-label");
    }
}

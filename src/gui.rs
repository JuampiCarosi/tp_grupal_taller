use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use gtk::builders::LabelBuilder;
use gtk::prelude::*;
use gtk::{self, Align};

use crate::io::leer_a_string;
use crate::tipos_de_dato::comandos::branch::{self, Branch};
use crate::tipos_de_dato::comandos::log::Log;
use crate::tipos_de_dato::logger::Logger;
use crate::utilidades_de_compresion::descomprimir_objeto;

fn branch_dialog(builder: gtk::Builder) {
    let branch_button: gtk::Button = builder.object("branch-button").unwrap();
    let dialog: gtk::MessageDialog = builder.object("branch").unwrap();

    dialog.set_position(gtk::WindowPosition::Center);

    branch_button.connect_clicked(move |_| {
        dialog.run();
        dialog.hide();
    });

    let confirm: gtk::Button = builder.object("confirm-branch").unwrap();
    let cancel: gtk::Button = builder.object("cancel-branch").unwrap();

    let builder_clone = builder.clone();
    confirm.connect_clicked(move |_| {
        let dialog: gtk::MessageDialog = builder_clone.object("branch").unwrap();
        let input: gtk::Entry = builder_clone.object("branch-input").unwrap();
        println!("branch name: {}", input.text());
        dialog.hide();
    });

    cancel.connect_clicked(move |_| {
        let dialog: gtk::MessageDialog = builder.object("branch").unwrap();
        let input: gtk::Entry = builder.object("branch-input").unwrap();
        input.set_text("");
        dialog.hide();
    });
}

fn select_branch(builder: gtk::Builder, rama_seleccionada: Arc<Mutex<String>>) {
    let select: gtk::ComboBoxText = builder.object("select-branch").unwrap();

    let binding = Branch::mostrar_ramas().unwrap();
    let branches = binding.split("\n");

    branches.for_each(|branch| {
        if branch == "" {
            return;
        }
        select.append_text(branch);
    });

    select.connect_changed(move |select| {
        let active = select.active_text().unwrap();

        *rama_seleccionada.lock().unwrap() = active.to_string();

        println!("active: {:?}", rama_seleccionada);
    });

    select.set_active(Some(0 as u32));
}

fn obtener_listas_de_commits(branch: &String) -> Result<Vec<String>, String> {
    let ruta = format!(".gir/refs/heads/{}", branch);
    let mut ultimo_commit = leer_a_string(Path::new(&ruta))?;

    if ultimo_commit.is_empty() {
        return Ok(Vec::new());
    }
    let mut historial_commits: Vec<String> = Vec::new();
    historial_commits.push(ultimo_commit.clone());
    loop {
        let contenido = descomprimir_objeto(ultimo_commit.clone())?;
        let siguiente_padre = Log::conseguir_padre_desde_contenido_commit(&contenido);
        if siguiente_padre.is_empty() {
            break;
        }
        historial_commits.push(ultimo_commit.clone());
        ultimo_commit = siguiente_padre.to_string();
    }
    Ok(historial_commits)
}

fn crear_label(string: &str) -> gtk::Label {
    gtk::Label::new(Some(string))
}

fn mostrar_log(builder: gtk::Builder, branch: Arc<Mutex<String>>) {
    let container: gtk::Box = builder.object("log-container").unwrap();

    let commits = obtener_listas_de_commits(&branch.lock().unwrap().clone()).unwrap();
    println!("commits: {:?}", commits);

    for commit in commits {
        let label = crear_label(&commit);
        container.pack_start(&label, true, true, 0);
    }
}

pub fn ejecutar() {
    if gtk::init().is_err() {
        println!("Failed to initialize GTK.");
        return;
    }

    let rama_seleccionada = Arc::new(Mutex::new(String::new()));

    let glade_src = include_str!("glade1.glade");
    let builder = gtk::Builder::from_string(glade_src);

    let window: gtk::Window = builder.object("home").unwrap();
    window.set_position(gtk::WindowPosition::Center);

    branch_dialog(builder.clone());
    select_branch(builder.clone(), rama_seleccionada.clone());
    mostrar_log(builder.clone(), rama_seleccionada.clone());

    window.show_all();

    gtk::main();
}

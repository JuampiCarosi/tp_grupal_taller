use std::path::{Path, PathBuf};
use std::rc::Rc;

use gtk::builders::LabelBuilder;
use gtk::glib::GString;
use gtk::prelude::*;
use gtk::{self, Align};

use crate::io::leer_a_string;
use crate::tipos_de_dato::comandos::branch::{self, Branch};
use crate::tipos_de_dato::comandos::checkout::Checkout;
use crate::tipos_de_dato::comandos::commit::Commit;
use crate::tipos_de_dato::comandos::log::Log;
use crate::tipos_de_dato::logger::Logger;
use crate::utilidades_de_compresion::descomprimir_objeto;

fn branch_dialog(builder: gtk::Builder, window: gtk::Window) {
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
        Branch::from(
            &mut vec![input.text().to_string()],
            Rc::new(Logger::new(PathBuf::from("log.txt")).unwrap()),
        )
        .unwrap()
        .ejecutar()
        .unwrap();

        select_branch(builder_clone.clone(), window.clone());
        input.set_text("");
        dialog.hide();
    });

    cancel.connect_clicked(move |_| {
        let dialog: gtk::MessageDialog = builder.object("branch").unwrap();
        let input: gtk::Entry = builder.object("branch-input").unwrap();
        input.set_text("");
        dialog.hide();
    });
}

fn select_branch(builder: gtk::Builder, window: gtk::Window) {
    let select: gtk::ComboBoxText = builder.object("select-branch").unwrap();
    let branch_actual = Commit::obtener_branch_actual().unwrap();
    select.remove_all();

    let binding = Branch::mostrar_ramas().unwrap();
    let branches = binding.split("\n");

    let mut i = 0;
    branches.for_each(|branch| {
        if branch == "" {
            return;
        }
        select.append_text(branch);
        if branch == branch_actual {
            select.set_active(Some(i));
        }
        i += 1;
    });

    select.connect_changed(move |a| {
        let logger = Rc::new(Logger::new(PathBuf::from("log.txt")).unwrap());

        let active = match a.active_text() {
            Some(text) => text,
            None => return,
        };

        mostrar_log(builder.clone(), active.to_string());
        Checkout::from(vec![active.to_string()], logger)
            .unwrap()
            .ejecutar()
            .unwrap();
        window.show_all();
    });
}

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

fn crear_label(string: &str) -> gtk::Label {
    gtk::Label::new(Some(string))
}

fn mostrar_log(builder: gtk::Builder, branch: String) {
    let container: gtk::Box = builder.object("log-container").unwrap();
    container.children().iter().for_each(|child| {
        container.remove(child);
    });

    let commits = obtener_listas_de_commits(&branch).unwrap();

    for commit in commits {
        let label = crear_label(&commit);
        container.add(&label);
    }
}

pub fn ejecutar() {
    if gtk::init().is_err() {
        println!("Failed to initialize GTK.");
        return;
    }

    let glade_src = include_str!("glade1.glade");
    let builder = gtk::Builder::from_string(glade_src);

    let window: gtk::Window = builder.object("home").unwrap();
    window.set_position(gtk::WindowPosition::Center);

    branch_dialog(builder.clone(), window.clone());
    select_branch(builder.clone(), window.clone());

    window.show_all();

    gtk::main();
}

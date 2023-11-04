use gtk::prelude::*;

use crate::utils::compresion::{descomprimir_objeto, descomprimir_objeto_gir};

fn estilar_log_info(builder: &gtk::Builder) {
    let text: gtk::TextView = builder.object("log-description").unwrap();

    let css_provider = gtk::CssProvider::new();
    css_provider
        .load_from_data(
            "
            textview  {
                font-size: 12px;
                font-family: monospace;
                background-color: #f6f5f4;
                border-radius: 8px;
            }

            text  {
              background-color: #DDDDDD;
              border-radius: 8px;
              border: 1px solid #9A9A9A;

          }
        "
            .as_bytes(),
        )
        .unwrap();

    let context = text.style_context();

    gtk::StyleContext::add_provider(
        &context,
        &css_provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    text.set_wrap_mode(gtk::WrapMode::Word);

    text.set_editable(false);
    text.set_pixels_above_lines(5);
    text.set_pixels_below_lines(5);
    text.set_pixels_inside_wrap(5);
    text.set_left_margin(5);
}

fn esconder_log_info(builder: &gtk::Builder) {
    let text: gtk::TextView = builder.object("log-description").unwrap();

    let css_provider = gtk::CssProvider::new();
    css_provider
        .load_from_data(
            "
          textview text {
              background-color: #f6f5f4;
              border-radius: 0px;
          }

       
      "
            .as_bytes(),
        )
        .unwrap();

    let context = text.style_context();

    gtk::StyleContext::add_provider(
        &context,
        &css_provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}

pub fn render(builder: &gtk::Builder, commit: Option<&str>) {
    let log_info: gtk::TextBuffer = builder.object("log-info").unwrap();
    if let Some(commit) = commit {
        let contenido = descomprimir_objeto_gir(commit.to_string()).unwrap();
        let contenido_split = contenido.split('\0').collect::<Vec<&str>>();
        log_info.set_text(contenido_split[1]);

        estilar_log_info(builder);
    } else {
        log_info.set_text("");
        esconder_log_info(builder);
    }
}

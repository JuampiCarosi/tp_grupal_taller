pub mod err_comunicacion;
pub mod servidor {
    pub mod receive_pack;
    pub mod server;
    pub mod upload_pack;
}
pub mod tipos_de_dato {
    pub mod comando;
    pub mod comunicacion;
    pub mod config;
    pub mod logger;
    pub mod objeto;
    pub mod packfile;
    pub mod visualizaciones;
    pub mod objetos {
        pub mod blob;
        pub mod commit;
        pub mod tree;
    }
    pub mod comandos {
        pub mod add;
        pub mod branch;
        pub mod cat_file;
        pub mod checkout;
        pub mod clone;
        pub mod commit;
        pub mod fetch;
        pub mod hash_object;
        pub mod init;
        pub mod log;
        pub mod ls_files;
        pub mod ls_tree;
        pub mod merge;
        pub mod pull;
        pub mod push;
        pub mod remote;
        pub mod rm;
        pub mod show_ref;
        pub mod status;
        pub mod version;
        pub mod write_tree;
    }
}
pub mod gui;

pub mod utils {
    pub mod compresion;
    pub mod gir_config;
    pub mod index;
    pub mod io;
    pub mod objects;
    pub mod path_buf;
    pub mod ramas;
    pub mod strings;
}

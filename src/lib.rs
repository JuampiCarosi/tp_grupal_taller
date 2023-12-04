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
    pub mod conflicto;
    pub mod lado_conflicto;
    pub mod logger;
    pub mod objeto;
    pub mod packfile;
    pub mod referencia;
    pub mod region;
    pub mod tipo_diff;
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
        pub mod check_ignore;
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
        pub mod rebase;
        pub mod remote;
        pub mod rm;
        pub mod set_upstream;
        pub mod show_ref;
        pub mod status;
        pub mod tag;
        pub mod version;
        pub mod write_tree;
    }
}
pub mod gui;

pub mod utils {
    pub mod compresion;
    pub mod fase_descubrimiento;
    pub mod gir_config;
    pub mod index;
    pub mod io;
    pub mod objects;
    pub mod path_buf;
    pub mod ramas;
    pub mod strings;
    pub mod tags;
    pub mod testing;
}

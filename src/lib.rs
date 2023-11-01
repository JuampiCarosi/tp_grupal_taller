pub mod tipos_de_dato {
    pub mod comando;
    pub mod config;
    pub mod logger;
    pub mod objeto;
    pub mod visualizaciones;
    pub mod objetos {
        pub mod blob;
        pub mod tree;
    }
    pub mod comandos {
        pub mod add;
        pub mod branch;
        pub mod cat_file;
        pub mod checkout;
        pub mod commit;
        pub mod hash_object;
        pub mod init;
        pub mod log;
        pub mod remote;
        pub mod rm;
        pub mod status;
        pub mod version;
        pub mod write_tree;
    }
}
pub mod gui;
pub mod io;
pub mod utilidades_de_compresion;
pub mod utilidades_index;
pub mod utilidades_path_buf;

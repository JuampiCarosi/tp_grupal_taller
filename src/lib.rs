pub mod comunicacion;
pub mod err_comunicacion;
pub mod io;
pub mod packfile;
pub mod server;
pub mod tipos_de_dato {
    pub mod comando;
    pub mod config;
    pub mod logger;
    pub mod objeto;
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
        pub mod merge;
        pub mod push;
        pub mod remote;
        pub mod rm;
        pub mod status;
        pub mod version;
        pub mod write_tree;
    }
}
pub mod receive_pack;
pub mod upload_pack;
pub mod utilidades_de_compresion;
pub mod utilidades_index;
pub mod utilidades_path_buf;
pub mod utilidades_strings;

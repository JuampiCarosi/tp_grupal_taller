pub mod server;
pub mod err_comunicacion;
pub mod io;
pub mod comunicacion;
pub mod tipos_de_dato {
    pub mod comando;
    pub mod logger;
    pub mod objeto;
    pub mod visualizaciones;
    pub mod objetos {
        pub mod blob;
        pub mod tree;
    }
    pub mod comandos {
        pub mod cat_file;
        pub mod hash_object;
        pub mod init;
        pub mod update_index;
        pub mod version;
    }
}

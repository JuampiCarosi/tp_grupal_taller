pub mod server;
pub mod err_comunicacion;
pub mod io;
pub mod comunicacion;
pub mod packfile;
pub mod tipos_de_dato {
    pub mod comando;
    pub mod logger;
    pub mod objeto;
    pub mod utilidades_index;
    pub mod visualizaciones;
    pub mod objetos {
        pub mod blob;
        pub mod tree;
    }
    pub mod comandos {
        pub mod add;
        pub mod branch;
        pub mod cat_file;
        pub mod hash_object;
        pub mod init;
        pub mod rm;
        pub mod version;
    }
}
pub mod utilidades_de_compresion;
pub mod utilidades_path_buf;
pub mod utilidades_strings;
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
        pub mod add;
        pub mod cat_file;
        pub mod hash_object;
        pub mod init;
        pub mod version;
    }
}
pub mod io;
pub mod utilidades_de_compresion;
pub mod utilidades_path_buf;

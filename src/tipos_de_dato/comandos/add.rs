use std::{path::PathBuf, rc::Rc};

use crate::tipos_de_dato::{
    logger::Logger,
    objeto::Objeto,
    utilidades_index::{crear_index, escribir_index, leer_index},
};

pub struct Add {
    logger: Rc<Logger>,
    ubicaciones: Vec<PathBuf>,
    index: Vec<Objeto>,
}

impl Add {
    pub fn from(args: Vec<String>, logger: Rc<Logger>) -> Result<Add, String> {
        crear_index();
        let index = leer_index()?;
        let ubicaciones = args.iter().map(PathBuf::from).collect::<Vec<PathBuf>>();

        Ok(Add {
            logger,
            ubicaciones,
            index,
        })
    }

    pub fn ejecutar(&mut self) -> Result<String, String> {
        self.logger.log("Ejecutando update-index".to_string());

        for ubicacion in self.ubicaciones.clone() {
            let nuevo_objeto = Objeto::from_directorio(ubicacion.clone(), None)?;

            let indice = self.index.iter().position(|x| match x {
                Objeto::Blob(blob) => blob.ubicacion == ubicacion,
                Objeto::Tree(tree) => tree.directorio == ubicacion,
            });

            if let Some(i) = indice {
                self.index[i] = nuevo_objeto;
            } else {
                self.index.push(nuevo_objeto);
            }
        }

        escribir_index(self.logger.clone(), &self.index)?;
        Ok("".to_string())
    }
}

#[cfg(test)]

mod test {
    use std::{io::Write, path::PathBuf, rc::Rc};

    use crate::{
        io,
        tipos_de_dato::{comandos::add::Add, logger::Logger, objeto::Objeto},
    };

    fn create_test_file() {
        let mut file = std::fs::File::create("test_file.txt").unwrap();
        let _ = file.write_all(b"test file");
    }

    fn modify_test_file() {
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .open("test_file.txt")
            .unwrap();
        let _ = file.write_all(b"test file modified");
    }

    fn clear_index() {
        let _ = std::fs::remove_file("./.gir/index");
    }

    #[test]
    fn test01_archivo_vacio_se_llena_con_objeto_agregado() {
        clear_index();
        create_test_file();
        let logger = Rc::new(Logger::new(PathBuf::from("tmp/add_test01")).unwrap());
        let ubicacion = "test_file.txt".to_string();
        let mut add = Add::from(vec![ubicacion], logger).unwrap();

        add.ejecutar().unwrap();

        assert_eq!(add.index.len(), 1);

        let file = io::leer_a_string("./.gir/index").unwrap();
        assert_eq!(
            file,
            "100644 bdf08de0f3095da5030fecd9bafc0b00c1aced7c test_file.txt\n"
        );
    }

    #[test]
    fn test02_archivo_con_objeto_actualiza_el_objeto() {
        modify_test_file();
        let logger = Rc::new(Logger::new(PathBuf::from("tmp/add_test02")).unwrap());
        let ubicacion = "test_file.txt".to_string();
        let mut add = Add::from(vec![ubicacion], logger).unwrap();

        add.ejecutar().unwrap();

        assert_eq!(add.index.len(), 1);

        let objeto = &add.index[0];
        if let Objeto::Blob(blob) = objeto {
            assert_eq!(blob.nombre, "test_file.txt");
            assert_eq!(blob.hash, "678e12dc5c03a7cf6e9f64e688868962ab5d8b65");
        }

        let file = io::leer_a_string("./.gir/index").unwrap();
        assert_eq!(
            file,
            "100644 678e12dc5c03a7cf6e9f64e688868962ab5d8b65 test_file.txt\n"
        );
    }

    #[test]
    fn test03_agregar_un_objeto_en_un_directorio() {
        clear_index();

        let logger = Rc::new(Logger::new(PathBuf::from("tmp/add_test03")).unwrap());

        let path = "test_dir/objetos/archivo.txt".to_string();
        let mut add = Add::from(vec![path], logger.clone()).unwrap();
        add.ejecutar().unwrap();

        let file = io::leer_a_string("./.gir/index").unwrap();

        assert_eq!(
            file,
            "40000 1f67151c34d6b33ec1a98fdafef8b021068395a0 test_dir\n"
        );
    }

    #[test]
    fn test04_archivo_con_objetos_agrega_nuevos_objetos() {
        clear_index();
        let logger = Rc::new(Logger::new(PathBuf::from("tmp/add_test04")).unwrap());
        let ubicacion = "test_file.txt".to_string();

        let mut add = Add::from(vec![ubicacion], logger.clone()).unwrap();
        add.ejecutar().unwrap();

        let ubicacion = "test_dir/objetos/archivo.txt".to_string();

        let mut add = Add::from(vec![ubicacion], logger.clone()).unwrap();
        add.ejecutar().unwrap();

        assert_eq!(add.index.len(), 2);

        let objeto = &add.index[0];

        if let Objeto::Blob(blob) = objeto {
            assert_eq!(blob.nombre, "test_file.txt");
            assert_eq!(blob.hash, "678e12dc5c03a7cf6e9f64e688868962ab5d8b65");
        }

        let objeto = &add.index[1];

        if let Objeto::Blob(blob) = objeto {
            assert_eq!(blob.nombre, "archivo.txt");
            assert_eq!(blob.hash, "2b824e648965b94c6c6b3dd0702feb91f699ed62");
        }

        let file = io::leer_a_string("./.gir/index").unwrap();

        assert_eq!(
            file,
            "40000 1f67151c34d6b33ec1a98fdafef8b021068395a0 test_dir\n100644 678e12dc5c03a7cf6e9f64e688868962ab5d8b65 test_file.txt\n"
        );
    }

    #[test]
    fn test05_agregar_un_directorio_al_index() {
        clear_index();
        let logger = Rc::new(Logger::new(PathBuf::from("tmp/add_test05")).unwrap());

        let path = "test_dir/muchos_objetos".to_string();
        let mut add = Add::from(vec![path], logger.clone()).unwrap();
        add.ejecutar().unwrap();

        let file = io::leer_a_string("./.gir/index").unwrap();

        assert_eq!(
            file,
            "40000 64aec41173e6ef51f8918e665fb5dfc5247ae08a test_dir\n"
        );
    }

    #[test]
    fn test06_editar_hijo_actualiza_padre() {
        clear_index();

        let logger = Rc::new(Logger::new(PathBuf::from("tmp/add_test06")).unwrap());

        let archivo_1 = "test_dir/muchos_objetos/archivo.txt".to_string();
        let mut add = Add::from(vec![archivo_1], logger.clone()).unwrap();
        add.ejecutar().unwrap();

        let file = io::leer_a_string("./.gir/index").unwrap();

        assert_eq!(
            file,
            "40000 74f21726ba8d24ac75264d20a2042e4901694e70 test_dir\n"
        );

        let archivo_2 = "test_dir/muchos_objetos/archivo_copy.txt".to_string();

        let mut add = Add::from(vec![archivo_2], logger.clone()).unwrap();
        add.ejecutar().unwrap();

        let file = io::leer_a_string("./.gir/index").unwrap();

        assert_eq!(
            file,
            "40000 64aec41173e6ef51f8918e665fb5dfc5247ae08a test_dir\n"
        );
    }

    #[test]
    fn test07_agregar_dos_archivos() {
        clear_index();
        let logger = Rc::new(Logger::new(PathBuf::from("tmp/add_test07")).unwrap());
        let ubicacion = "test_file.txt".to_string();

        let ubicacion2 = "test_dir/objetos/archivo.txt".to_string();

        let mut add = Add::from(vec![ubicacion, ubicacion2], logger.clone()).unwrap();
        add.ejecutar().unwrap();

        assert_eq!(add.index.len(), 2);

        let objeto = &add.index[0];

        if let Objeto::Blob(blob) = objeto {
            assert_eq!(blob.nombre, "test_file.txt");
            assert_eq!(blob.hash, "678e12dc5c03a7cf6e9f64e688868962ab5d8b65");
        }

        let objeto = &add.index[1];

        if let Objeto::Blob(blob) = objeto {
            assert_eq!(blob.nombre, "archivo.txt");
            assert_eq!(blob.hash, "2b824e648965b94c6c6b3dd0702feb91f699ed62");
        }

        let file = io::leer_a_string("./.gir/index").unwrap();

        assert_eq!(
            file,
            "40000 1f67151c34d6b33ec1a98fdafef8b021068395a0 test_dir\n100644 678e12dc5c03a7cf6e9f64e688868962ab5d8b65 test_file.txt\n"
        );
    }
}

use std::net::{TcpListener, TcpStream};
use std::io::{Write, Read, BufRead};
use std::env;
use std::path::PathBuf;
use std::fs;
pub struct Comunicacion { 
    listener: TcpListener,
    stream: TcpStream,
    dir: String,
}

impl Comunicacion { 

    pub fn new(listener: TcpListener) -> Comunicacion {
        let (stream, _) = listener.accept().unwrap();
        let dir = match env::current_dir() {
            Ok(current_dir) => current_dir.to_str().unwrap().to_string(),
            Err(_) => '.'.to_string(),
        };
        Comunicacion { listener, stream, dir }
    }

    pub fn procesar_datos(&mut self) -> std::io::Result<()> {
        // lee primera parte, 4 bytes en hexadecimal indican el largo del stream
        let mut length_bytes = [0; 4];
        self.stream.read_exact(&mut length_bytes)?;
        // largo de bytes a str
        let length_str = std::str::from_utf8(&length_bytes).unwrap(); 
        // transforma str a u32
        let length = u32::from_str_radix(length_str, 16).unwrap();
        println!("length: {:?}", length);
    
        // lee el resto del stream
        let mut data = vec![0; (length - 4) as usize];
        self.stream.read_exact(&mut data)?;
        let line = String::from_utf8(data).unwrap();
        println!("Received: {:?}", line);
        // println!("length: {:?}", Self::calcular_largo_hex(&line));
        self.parse_line(&line);
        // stream.write(line.as_bytes())?;
        
        Ok(())
    }
    pub fn calcular_largo_hex(line: &str) -> String {
        let largo = line.len() + 4; // el + 4 es por los 4 bytes que indican el largo
        let largo_hex = format!("{:x}", largo);
        format!("{:0>4}", largo_hex)
    }

    fn parse_line(&mut self, line: &str) {
        let req: Vec<&str> = line.split_whitespace().collect();
        // veo si es un comando git
       
        // print!("req: {:?}", req[0]);
        match req[0] {
            "git-upload-pack" => {
                println!("git-upload-pack");
                let args: Vec<_> = req[1].split('\0').collect();
                // for i in a {
                //     println!("i: {:?}", i);
                // }
                let mut path = PathBuf::from(self.dir.clone() + args[0] + "refs/");
                println!("path: {:?}", path);
                if path.exists() {
                    self.send_capabilities_refs(&mut path, "heads/");
                    self.send_capabilities_refs(&mut path, "tags/");
                } else {
                    println!("no existe el repo");
                }
            },
            _ =>    println!("No se reconoce el comando"),
        }
    }
    
    fn send_capabilities_refs(&mut self, path: &mut PathBuf, dir: &str) -> std::io::Result<()> {
        // let head_ref = self.get_head_ref(path.clone().join("HEAD"));
        let head_dir = fs::read_dir(path.clone().join(dir))?;
        for archivo in head_dir {
            match archivo {
                Ok(archivo) => {
                    let path = archivo.path();
                    // let nombre_archivo = path.file_name().unwrap().to_str().unwrap();
                    let archivo = std::fs::File::open(path.clone())?; 
                    let mut contents = String::new();            
                    std::io::BufReader::new(archivo).read_line(&mut contents)?;
                    // let metadata = archivo.metadata()?;
                    // let file_size = metadata.len();
                    println!("{}", format!("{} {}", contents.trim(), path.to_str().unwrap()));
                }
                Err(error) => {
                    eprintln!("Error leyendo directorio: {}", error);
                }
            }
        }
        Ok(())

        // path.push("info");
        // path.push("refs");
        // path.push("heads");
        // path.push("master");
        // println!("path: {:?}", path);
        // let mut file = std::fs::File::open(path).unwrap();
        // let mut contents = String::new();
        // file.read_to_string(&mut contents).unwrap();
        // println!("contents: {:?}", contents);
        // let mut length_hex = Self::calcular_largo_hex(&contents);
        // let mut response = length_hex + &contents;
        // println!("response: {:?}", response);
        // self.stream.write(response.as_bytes()).unwrap();
    }

    // fn get_head_ref(&self, path: PathBuf) -> std::io::Result<String> {
    //     if !path.is_file() {
    //         return Err(std::io::Error::new(std::io::ErrorKind::NotFound, "No existe HEAD"));
    //     }
        
    //     let file = std::fs::File::open(path)?;
    //     let mut contents = String::new();            
    //     std::io::BufReader::new(file).read_line(&mut contents)?;
    //     let mut head_ref: Vec<&str> = contents.split_whitespace().collect();
                    
    // }
}
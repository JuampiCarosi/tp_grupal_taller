use taller::comando::Comando;
use std::io;

fn main() -> io::Result<()> {
    loop {
        let mut buf = String::new();
        let _ = io::stdin().read_line(&mut buf)?;
        let comando = match Comando::new(buf.split_whitespace().map(String::from).collect()){
            Ok(cmd) => {cmd},
            Err(error) => {
                eprintln!("Error: {}", error);
                continue;
            }
        };    

        match comando.ejecutar() {
            Ok(_) => {
                println!("Se ejeciuto el comando: {}", buf);
            }
            Err(error) => {
                eprintln!("Error: {}", error);
            }
        }
    }
}



// fn main() -> Result<(), String> {
//     let arguments = std::env::args().collect::<Vec<String>>();
//     let (_, args) = arguments.split_first().unwrap();
//     let comando = Comando::new(args.to_vec())?;
//     comando.ejecutar()?;

//     // let concatenated_arguments = args.to_vec().join("");

//     // let command = command_parser(&concatenated_arguments);
//     // command.execute();

//     Ok(())
// }

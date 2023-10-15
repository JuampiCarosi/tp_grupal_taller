use taller::comando::Comando;


fn main() -> Result<(), String> {
    let comando = Comando::new(std::env::args().collect())?;
    comando.ejecutar()?;
    Ok(())
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

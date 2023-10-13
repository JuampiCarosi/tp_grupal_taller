//use taller::command::command_parser;
use taller::comando::Comando;

fn main() -> Result<(), String> {
    let comando = Comando::new(std::env::args().collect())?;
    comando.ejecutar();
    // let arguments = std::env::args().collect::<Vec<String>>();
    // let (_, args) = arguments.split_first().unwrap();

    // let concatenated_arguments = args.to_vec().join("");

    // let command = command_parser(&concatenated_arguments);
    // command.execute();

    Ok(())
}

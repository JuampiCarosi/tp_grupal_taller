use taller::command::{command_parser, Command};

fn main() {
    let binding = std::env::args().collect::<Vec<String>>();
    let (_, args) = binding.split_first().unwrap();

    let argss = args.to_vec().join("");

    let command = command_parser(&argss);
    command.execute();
}

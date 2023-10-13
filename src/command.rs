use crate::{unknown_command::UnknownCommand, version_command::VersionCommand, init_command::InitCommand};

pub trait Command {
    fn execute(&self);
    // fn with_args(args: Vec<String>) -> Self
    // where
    //     Self: Sized;
}

pub fn command_parser(input: &Vec<String>) -> Box<dyn Command> {
    let cmd = format!("{} {}", input[0], input[1]);


    match cmd.as_str() {
        "git version" => Box::new(VersionCommand::from(Vec::new())),
        "git init" => Box::new(InitCommand::from(Vec::new())),
        _ => Box::new(UnknownCommand::from(Vec::new())),
    }
}

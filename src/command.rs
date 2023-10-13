use crate::{unknown_command::UnknownCommand, version_command::VersionCommand};

pub trait Command {
    fn execute(&self);
    fn with_args(args: Vec<String>) -> Self
    where
        Self: Sized;
}

pub fn command_parser(input: &str) -> Box<dyn Command> {
    match input {
        "version" => Box::new(VersionCommand::with_args(Vec::new())),
        _ => Box::new(UnknownCommand::with_args(Vec::new())),
    }
}

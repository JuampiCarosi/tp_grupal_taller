use crate::command::Command;

pub struct UnknownCommand {}

impl Command for UnknownCommand {
    fn execute(&self) {
        println!("Unknown command");
    }

    // fn with_args(_: Vec<String>) -> UnknownCommand {
    //     UnknownCommand {}
    // }
}

impl From<Vec<String>>for UnknownCommand {
    fn from(_: Vec<String>) -> Self {
        UnknownCommand {}
    }
}
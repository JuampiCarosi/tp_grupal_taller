use crate::command::Command;

pub struct VersionCommand {}

impl Command for VersionCommand {
    fn execute(&self) {
        println!("Version 0.1.0");
    }

    fn with_args(_: Vec<String>) -> VersionCommand {
        VersionCommand {}
    }
}

use std::{thread, time::Duration};

use taller::logger::Logger;

fn main() {
    let logger = Logger::new();

    let logger1 = logger.clone();
    thread::spawn(move || {
        logger1.log("Thread 1".to_string()).unwrap();
    });
    let logger2 = logger.clone();
    thread::spawn(move || {
        logger2.log("Thread 1".to_string()).unwrap();
    });
    let logger3 = logger.clone();
    thread::spawn(move || {
        logger3.log("Thread 1".to_string()).unwrap();
    });

    thread::sleep(Duration::from_secs(2));
}

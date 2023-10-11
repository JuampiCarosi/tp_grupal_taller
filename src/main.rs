use std::{sync::Arc, thread};

use taller::logger::Logger;

fn main() {
    let logger = Arc::new(Logger::new().unwrap());

    let logger1 = logger.clone();
    let handle_1 = thread::spawn(move || {
        logger1.log("Thread 1 saluda".to_string());
    });

    let logger2 = logger.clone();
    let handle_2 = thread::spawn(move || {
        logger2.log("Thread 2 saluda".to_string());
    });

    let logger3 = logger.clone();
    let handle_3 = thread::spawn(move || {
        logger3.log("Thread 3 saluda".to_string());
    });

    handle_1.join().unwrap();
    handle_2.join().unwrap();
    handle_3.join().unwrap();
}

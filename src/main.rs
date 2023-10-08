use std::{
    sync::mpsc,
    thread};

use taller::logger::Logger;

fn main() {
    let (tx, rx) = mpsc::channel();

    let log_thread_handle= thread::spawn(move ||{
        let logger = Logger::new(rx);
        logger.start_logging();
    });
        
    let tx1 = tx.clone();    
    let tx2 = tx.clone();    

    let handle_1 = thread::Builder::new().name("Thread1".into()).spawn(move || {
        tx1.send("Thread 1".to_string()).unwrap();       
    });
    let handle_2 = thread::Builder::new().name("Thread1".into()).spawn(move || {
        tx.send("Thread 2".to_string()).unwrap();       
    });
    let handle_3 = thread::Builder::new().name("Thread 3".into()).spawn(move || {
        tx2.send("Thread 3".to_string()).unwrap();       
    });



    handle_1.unwrap().join().unwrap();
    handle_2.unwrap().join().unwrap();
    log_thread_handle.join().unwrap();
    handle_3.unwrap().join().unwrap();
}

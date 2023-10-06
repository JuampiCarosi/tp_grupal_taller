use std::{
    sync::{mpsc, Arc},
    thread,
};
pub struct Logger {
    tx: Option<mpsc::Sender<String>>,
    handle: Option<thread::JoinHandle<()>>,
}
impl Logger {
    pub fn new() -> Arc<Logger> {
        let (tx, rx) = mpsc::channel();

        let handle = thread::spawn(move || {
            println!("Logger started");
            for msg in rx {
                println!("Log received: {}", msg);
            }
        });

        Arc::new(Logger {
            tx: Some(tx),
            handle: Some(handle),
        })
    }

    pub fn log(&self, msg: String) -> Result<(), mpsc::SendError<String>> {
        match self.tx {
            Some(ref tx) => tx.send(msg),
            None => Err(mpsc::SendError(msg)),
        }
    }
}
// }

impl Drop for Logger {
    fn drop(&mut self) {
        self.tx.take();
        self.handle.take().unwrap().join().unwrap();
        println!("Logger dropped");
    }
}

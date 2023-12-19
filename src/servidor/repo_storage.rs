use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

#[derive(Clone)]
pub struct RepoStorage {
    pub repo_mutexes: Arc<Mutex<HashMap<String, Arc<Mutex<()>>>>>,
}

impl RepoStorage {
    pub fn new() -> Self {
        RepoStorage {
            repo_mutexes: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl Default for RepoStorage {
    fn default() -> Self {
        Self::new()
    }
}

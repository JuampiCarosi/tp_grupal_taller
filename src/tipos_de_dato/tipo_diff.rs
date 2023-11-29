#[derive(Debug, Clone)]

pub enum TipoDiff {
    Added(String),
    Removed(String),
    Unchanged(String),
}

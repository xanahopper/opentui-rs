#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Key {
    Static(&'static str),
    Owned(String),
    IndexPath(Vec<u32>),
}

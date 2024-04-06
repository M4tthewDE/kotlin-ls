
#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum Type {
    Nullable(String),
    NonNullable(String),
}

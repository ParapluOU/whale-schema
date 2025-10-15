use std::collections::hash_map::DefaultHasher;
use std::fmt;
use std::hash::{Hash, Hasher};

/// identifiers for the deduplication of graph structures that already use references (ID)
#[derive(Debug, Hash, PartialEq, Eq, Ord, PartialOrd, Clone, Copy)]
pub struct TypeHash(u64);

impl fmt::Display for TypeHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub trait GetTypeHash: Hash {
    fn id(&self) -> TypeHash {
        let mut s = DefaultHasher::new();
        self.hash(&mut s);
        TypeHash(s.finish())
    }
}

impl<T: Hash> GetTypeHash for T {}

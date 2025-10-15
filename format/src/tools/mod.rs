mod logging;
mod recursion;

pub use {logging::*, recursion::*};

// helper that used to be in Rust but was removed
pub fn default<T: Default>() -> T {
    T::default()
}

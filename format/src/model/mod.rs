mod attr;
mod comment;
mod duplicity;
mod element;
mod group;
mod prelude;
mod primitive;
mod refs;
pub mod restriction;
mod schema;
mod simpletype;
mod r#type;
mod typehash;

pub use {
    attr::*, comment::*, element::*, group::*, primitive::*, r#type::*, schema::*, simpletype::*,
    typehash::*,
};

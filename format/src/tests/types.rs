use crate::model::{GetTypeHash, PrimitiveType};
use std::collections::HashSet;
use strum::IntoEnumIterator;

#[test]
fn test_primitive_type_hashes_different() {
    let primitives = PrimitiveType::iter().collect::<Vec<_>>();
    let hashes = primitives.iter().map(|p| p.id()).collect::<HashSet<_>>();
    let names = primitives
        .iter()
        .map(|p| p.to_string())
        .collect::<HashSet<_>>();

    assert_eq!(primitives.len(), hashes.len());
    assert_eq!(primitives.len(), names.len());

    dbg!(names);
}

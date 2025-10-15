use crate::model::group::Group;
use crate::model::primitive::PrimitiveType;
use crate::model::simpletype::SimpleType;
use crate::model::typehash::TypeHash;
use crate::model::{GetTypeHash, Ref, SchemaObjId};
use crate::{default, model};
use enum_variant_macros::FromVariants;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

/// map ordered by type hash
pub type TypeMap<T> = HashMap<TypeHash, T>;

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum TypeVariant {
    Simple,
    Group,
}

/// owned type that does not neccesarily already exist in the schema,
/// though any referred subtypes and subelements should
#[derive(PartialEq, Eq, Clone, Debug)]
pub enum Type {
    Simple(SimpleType),
    Group(Group),
}

impl Type {
    pub fn static_string(s: String, schema: &model::Schema) -> Self {
        Self::Simple(SimpleType::static_string(&s, schema))
    }

    pub fn to_type_name(&self, schema: &model::Schema) -> String {
        match self {
            Type::Simple(prim) => prim.to_type_name(schema),
            Type::Group(group) => panic!("cant coerce anonymous group to Type name string"),
        }
    }
}

impl Hash for Type {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Type::Simple(simple) => simple.hash(state),
            Type::Group(group) => group.hash(state),
        }
    }
}

impl Default for Type {
    fn default() -> Self {
        Self::Simple(default())
    }
}

impl<T: Into<SimpleType>> From<T> for Type {
    fn from(value: T) -> Self {
        Self::Simple(value.into())
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Debug, FromVariants)]
pub enum TypeRef {
    Simple(Ref<SimpleType>),
    Group(Ref<Group>),
}

impl TypeRef {
    pub fn schema_object_id(&self) -> &SchemaObjId {
        match self {
            TypeRef::Simple(simple) => simple.schema_object_id(),
            TypeRef::Group(group) => group.schema_object_id(),
        }
    }

    pub fn typehash(&self, schema: &model::Schema) -> TypeHash {
        match self {
            TypeRef::Simple(simple) => simple.resolve(schema).id(),
            TypeRef::Group(group) => group.resolve(schema).id(),
        }
    }

    pub fn simpletype<'a>(&'a self, schema: &'a model::Schema) -> Option<&'a model::SimpleType> {
        match self {
            TypeRef::Simple(simple) => schema.get_simpletype(simple),
            TypeRef::Group(_) => None,
        }
    }

    pub fn grouptype<'a>(&'a self, schema: &'a model::Schema) -> Option<&'a Group> {
        match self {
            TypeRef::Simple(_) => None,
            TypeRef::Group(group) => schema.get_group(group),
        }
    }

    pub fn is_mixed_content(&self, schema: &model::Schema) -> bool {
        match self {
            TypeRef::Simple(simple) => false,
            TypeRef::Group(group) => *group.resolve(schema).mixed(),
        }
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub enum TypeBor<'a> {
    Simple(&'a SimpleType),
    Group(&'a Group),
}

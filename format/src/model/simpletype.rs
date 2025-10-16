use crate::ast::TypeRegex;
use crate::model::primitive::PrimitiveType;
use crate::model::restriction::SimpleTypeRestriction;
use crate::model::Ref;
use crate::{ast, model, tools::default};
use pseudonym::alias;
use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Eq, Debug, PartialEq, Clone, Hash)]
pub enum SimpleType {
    Derived {
        /// reference to base type to derive from
        base: Ref<SimpleType>,
        /// list of restrictions to apply to form new type
        restrictions: SimpleTypeRestriction,
        /// whether this simple type is abstract
        abstract_type: bool,
    },

    Builtin {
        name: PrimitiveType,
    },

    /// the "union" restriction allows for the definition of elements or attributes that can have
    /// values that match any one of the specified simple types.
    Union {
        /// list of members that are part of this union
        member_types: Vec<Ref<SimpleType>>,
    },

    /// the "list" restriction allows you to define elements or attributes that can
    /// contain a list of values of a specified simple type separated by a
    /// specified separator. It's typically used when you need to represent a
    /// list of values within an XML document.
    List {
        /// refer to either simpletype or com[plex type content to define items
        item_type: Ref<SimpleType>,

        /// Default Separator: If no separator is specified, whitespace is
        /// used as the default separator. However, you can explicitly define a separator
        /// using the <xs:list> element's separator attribute.
        separator: Option<String>,
    },
}

impl Default for SimpleType {
    fn default() -> Self {
        Self::Builtin {
            name: PrimitiveType::default(),
        }
    }
}

impl SimpleType {
    pub fn static_string(s: &impl ToString, schema: &model::Schema) -> Self {
        Self::Derived {
            base: schema
                .get_simpletype_ref(&model::PrimitiveType::String.into())
                .unwrap(),
            restrictions: SimpleTypeRestriction {
                enumeration: Some(vec![s.to_string()]),
                ..default()
            },
            abstract_type: false,
        }
    }

    pub fn static_number(n: &ast::Uint, schema: &model::Schema) -> Self {
        Self::Derived {
            base: schema
                .get_simpletype_ref(&model::PrimitiveType::Int.into())
                .unwrap(),
            restrictions: SimpleTypeRestriction {
                enumeration: Some(vec![n.value.to_string()]),
                ..default()
            },
            abstract_type: false,
        }
    }

    pub fn from_regex(regex: &ast::TypeRegex, schema: &model::Schema) -> Self {
        Self::Derived {
            base: schema
                .get_simpletype_ref(&model::PrimitiveType::String.into())
                .unwrap(),
            restrictions: SimpleTypeRestriction {
                pattern: Some(regex.value.clone()),
                ..default()
            },
            abstract_type: false,
        }
    }

    pub fn to_type_name(&self, schema: &model::Schema) -> String {
        match self {
            SimpleType::Derived { base, .. } => base.resolve(schema).to_type_name(schema),
            SimpleType::Builtin { name } => name.to_string(),
            SimpleType::Union { .. } => {
                todo!("cant get single type name for restriction that is union of types")
            }
            SimpleType::List { .. } => PrimitiveType::String.to_string(),
        }
    }

    pub fn dependent_on_refs(&self) -> Vec<&Ref<SimpleType>> {
        match self {
            SimpleType::Derived { base, .. } => vec![base],
            SimpleType::Builtin { .. } => vec![],
            SimpleType::Union { member_types } => member_types.iter().collect(),
            SimpleType::List {
                item_type,
                separator,
            } => vec![item_type],
        }
    }

    #[alias(is_non_referencing)]
    pub fn is_builtin(&self) -> bool {
        match self {
            SimpleType::Builtin { .. } => true,
            _ => false,
        }
    }

    pub fn is_derived(&self) -> bool {
        matches!(self, Self::Derived { .. })
    }

    pub fn restrictions(&self) -> Option<&SimpleTypeRestriction> {
        match self {
            SimpleType::Derived { restrictions, .. } => Some(restrictions),
            _ => None,
        }
    }
}

impl From<&ast::Primitive> for SimpleType {
    fn from(prim: &ast::Primitive) -> Self {
        Self::Builtin { name: prim.into() }
    }
}

/// convert to SimpleType from a literal string type
// impl From<&ast::AttrItemStr> for SimpleType {
//     fn from(item: &ast::AttrItemStr) -> Self {
//         // Self {
//         //     ty: PrimitiveType::String,
//         //     constraint: Some(item.value.clone()),
//         // }
//
//         todo!("pass in schema ot retrieve refs")
//     }
// }

impl From<PrimitiveType> for SimpleType {
    fn from(ty: PrimitiveType) -> Self {
        Self::Builtin { name: ty }
    }
}

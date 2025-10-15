use crate::ast::Primitive;
use crate::formats::fonto;
use crate::model::restriction::SimpleTypeRestriction;
use crate::model::PrimitiveType;
use serde::{Deserialize, Serialize};

pub type SimpleTypeRef = usize;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(tag = "variety")]
#[serde(rename_all = "camelCase")]
pub enum SimpleType {
    Derived {
        /// reference to base type to derive from
        base: SimpleTypeRef,
        /// list of restrictions to apply to form new type
        restrictions: SimpleTypeRestriction,
    },

    Builtin {
        #[serde(rename = "localName")]
        name: fonto::Primitive,
    },

    /// the "union" restriction allows for the definition of elements or attributes that can have
    /// values that match any one of the specified simple types.
    Union {
        /// list of members that are part of this union
        #[serde(rename = "memberTypes")]
        member_types: Vec<SimpleTypeRef>,
    },

    /// the "list" restriction allows you to define elements or attributes that can
    /// contain a list of values of a specified simple type separated by a
    /// specified separator. It's typically used when you need to represent a
    /// list of values within an XML document.
    List {
        /// refer to either simpletype or com[plex type content to define items
        #[serde(rename = "itemType")]
        item_type: SimpleTypeRef,

        /// Default Separator: If no separator is specified, whitespace is
        /// used as the default separator. However, you can explicitly define a separator
        /// using the <xs:list> element's separator attribute.
        separator: Option<String>,
    },
}

impl SimpleType {
    pub fn validate_refs(&self, schema: &fonto::Schema) -> anyhow::Result<()> {
        match self {
            SimpleType::Derived { base, .. } => schema.assert_simpletype_idx(*base),
            SimpleType::Builtin { name } => Ok(()),
            SimpleType::Union { member_types } => {
                for member in member_types {
                    schema.assert_simpletype_idx(*member)?;
                }
                Ok(())
            }
            SimpleType::List { item_type, .. } => schema.assert_simpletype_idx(*item_type),
        }
    }
}

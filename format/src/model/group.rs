use crate::model::attr::Attributes;
use crate::model::element::Element;
use crate::model::Ref;
use crate::{ast, model};
use derive_builder::Builder;
use derive_getters::Getters;
use enum_variant_macros::FromVariants;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

/// group of elements in some order
#[derive(Debug, Hash, PartialEq, Eq, Clone, Builder, Getters)]
pub struct Group {
    /// block level defined attributes that are to be merged with
    /// element-level attributes
    #[builder(default)]
    attributes: Attributes,

    ///
    #[builder(default)]
    #[builder(setter(into))]
    ty: GroupType,

    /// whether the group allows mixed content (plain text nodes in between elements)
    #[builder(default)]
    mixed: bool,

    /// todo: probably needs to support more than only elements
    /// probably also needs control flow objects like groups themselves
    #[builder(default)]
    items: Vec<GroupItem>,
}

/// group of elements in some order
#[derive(Debug, Hash, PartialEq, Eq, Clone, FromVariants)]
pub enum GroupItem {
    Element(Ref<Element>),
    Group(Ref<Group>),
}

#[derive(Debug, Hash, Clone, Copy, PartialEq, Eq, strum_macros::Display)]
pub enum GroupType {
    /// <xs:sequence>
    Sequence,
    /// <xs:choice>
    Choice,
    /// <xs:all>
    All,
}

impl Default for GroupType {
    fn default() -> Self {
        Self::Sequence
    }
}

impl From<&ast::BlockMods> for GroupType {
    fn from(ast: &ast::BlockMods) -> Self {
        if ast.is_all() {
            Self::All
        } else if ast.is_choice() {
            Self::Choice
        } else {
            Self::Sequence
        }
    }
}

impl Group {
    pub fn contains_element(&self, element: &Ref<model::Element>, schema: &model::Schema) -> bool {
        self.items.iter().any(|item| match item {
            GroupItem::Element(e) => e == element,
            GroupItem::Group(g) => g.resolve(schema).contains_element(element, schema),
        })
    }
}

use crate::model;
use crate::model::attr::Attributes;
use crate::model::duplicity::Duplicity;
use crate::model::r#type::Type;
use crate::model::{Comment, Ref, TypeRef};
use derive_builder::Builder;
use derive_getters::Getters;

#[derive(Debug, Hash, PartialEq, Eq, Clone, Builder, Getters)]
pub struct Element {
    name: String,

    /// element level defined attributes that are to be merged with
    /// block-level attributes
    #[builder(default)]
    attributes: Attributes,

    /// modifier indicating whether the element is optional,
    /// should be repeated, etc
    #[builder(default)]
    duplicity: Duplicity,

    /// we need to store the type as a reference so we can resolve it
    /// because if we want this to be an owned type, it means
    /// the compiler has to recursively walk the tree to create the type,
    /// but because types can be recursive it will loop and never resolve
    typing: TypeRef,

    /// comments associated with this attribute
    #[builder(default)]
    comments: Vec<Comment>,
}

impl Element {
    pub fn is_mixed_content(&self, schema: &model::Schema) -> bool {
        self.typing().is_mixed_content(schema)
    }

    pub fn is_local(&self, schema: &model::Schema) -> bool {
        let selfref = schema.get_element_ref(self).expect("element not found");

        for group in schema.types_group().values() {
            if group.contains_element(&selfref, schema) {
                return true;
            }
        }

        false
    }

    pub fn min_occurs(&self) -> usize {
        self.duplicity.min_occurs()
    }

    pub fn max_occurs(&self) -> Option<usize> {
        self.duplicity.max_occurs()
    }

    /// merge the attributes on the group type with the element's own attributes
    pub fn group_merged_attributes(&self, schema: &model::Schema) -> Attributes {
        match &self.typing {
            TypeRef::Simple(_) => self.attributes.clone(),
            TypeRef::Group(gr) => gr
                .resolve(schema)
                .attributes()
                .clone()
                .merge(self.attributes.clone()),
        }
    }
}

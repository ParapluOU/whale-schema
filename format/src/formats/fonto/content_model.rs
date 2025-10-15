use crate::formats::{AnyAttrValidation, Occurs};
use derive_builder::Builder;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "type")]
pub enum ContentModel {
    Sequence {
        items: Vec<ContentModel>,
        #[serde(rename = "maxOccurs")]
        max_occurs: Option<Occurs>,
        #[serde(rename = "minOccurs")]
        min_occurs: Option<Occurs>,
    },
    Choice {
        items: Vec<ContentModel>,
        #[serde(rename = "maxOccurs")]
        max_occurs: Option<Occurs>,
        #[serde(rename = "minOccurs")]
        min_occurs: Option<Occurs>,
    },
    All {
        items: Vec<ContentModel>,
    },
    /// element that only exists within the context of a sequence/choice/all item (nested)
    LocalElement {
        #[serde(rename = "elementRef")]
        element_ref: usize,

        #[serde(rename = "maxOccurs")]
        max_occurs: Option<Occurs>,
        #[serde(rename = "minOccurs")]
        min_occurs: Option<Occurs>,
    },
    Element {
        #[serde(rename = "localName")]
        name: String,

        #[serde(rename = "namespaceURI")]
        namespace_uri: Option<String>,

        #[serde(rename = "maxOccurs")]
        max_occurs: Option<Occurs>,
        #[serde(rename = "minOccurs")]
        min_occurs: Option<Occurs>,
    },
    Empty {
        #[serde(rename = "maxOccurs")]
        max_occurs: Option<Occurs>,
        #[serde(rename = "minOccurs")]
        min_occurs: Option<Occurs>,
    },
    Any {
        #[serde(rename = "processContents")]
        process_contents: Option<AnyAttrValidation>,

        #[serde(rename = "disallowedNamespaceNames")]
        disallowed_namespace_names: Option<Vec<String>>,
    },
}

impl ContentModel {
    pub fn is_empty(&self) -> bool {
        matches!(self, Self::Empty { .. })
    }

    pub fn validate_refs(&self, schema: &super::Schema) -> anyhow::Result<()> {
        match self {
            ContentModel::Sequence { items, .. } => {
                for item in items {
                    item.validate_refs(schema)?;
                }
            }
            ContentModel::Choice { items, .. } => {
                for item in items {
                    item.validate_refs(schema)?;
                }
            }
            ContentModel::All { items, .. } => {
                for item in items {
                    item.validate_refs(schema)?;
                }
            }
            ContentModel::LocalElement { element_ref, .. } => {
                schema.assert_local_element_idx(*element_ref)?;
            }
            ContentModel::Element { .. } => {}
            ContentModel::Empty { .. } => {}
            ContentModel::Any { .. } => {}
        }
        Ok(())
    }
}

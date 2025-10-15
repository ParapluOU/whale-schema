use crate::formats::fonto;
use anyhow::Context;
use derive_builder::Builder;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Builder, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Element {
    /// offset into the content models array
    #[builder(default)]
    content_model_ref: usize,

    /// like NISO-STS 'cbytes'
    /// if the SimpleType is set, the content_model_ref has to refer to the Empty variant
    #[builder(default)]
    simple_type_ref: Option<usize>,

    /// offset into the attribute definitions array
    #[builder(default)]
    attribute_refs: Vec<usize>,

    #[serde(rename = "localName")]
    name: String,

    #[serde(rename = "namespaceURI")]
    #[builder(default)]
    namespace_uri: Option<String>,

    #[serde(default)]
    #[builder(default)]
    is_mixed: bool,

    #[serde(default)]
    #[builder(default)]
    is_abstract: bool,

    /// configuration on how to deal with wildcard attributes
    #[builder(default)]
    any_attribute: Option<fonto::AnyAttrConf>,

    #[serde(default)]
    #[builder(default)]
    min_occurs: Option<Occurs>,

    #[serde(default)]
    #[builder(default)]
    max_occurs: Option<Occurs>,
}

impl Element {
    pub fn validate_refs(&self, schema: &super::Schema) -> anyhow::Result<()> {
        // validate ContentModel
        schema
            .assert_content_model_idx(self.content_model_ref)
            .context(format!("#{}", &self.name))?;

        // if a SimpleType is defined, validate it.
        // but as far as I can tell, whenever the type is validated
        // by a simple type, the content model should be empty.
        if let Some(str) = self.simple_type_ref {
            schema
                .assert_simpletype_idx(str)
                .context(format!("#{}", &self.name))?;

            assert_eq!(schema.get_content_model_empty_idx(), self.content_model_ref,
                       "when a SimpleType is set as model for an element, the ContentModel shoujld be set to the Empty variant");
        }

        // validate attributes
        for attr_ref in &self.attribute_refs {
            schema.assert_attribute_idx(*attr_ref)?;
        }
        Ok(())
    }
}

#[derive(Copy, Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Occurs(usize);

impl Default for Occurs {
    fn default() -> Self {
        Self(1)
    }
}

impl From<usize> for Occurs {
    fn from(occurs: usize) -> Self {
        Self(occurs)
    }
}

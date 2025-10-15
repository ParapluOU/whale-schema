use crate::export::FontoDefinitionIdx;
use crate::formats::fonto;
use crate::formats::fonto::version::FontoSchemaCompilerVersion;
use derive_builder::Builder;
use derive_getters::Getters;
use serde::*;
use std::path::Path;

/// representation of Fonto's JSON-based schema format
#[derive(Debug, Serialize, Deserialize, Builder, Clone, Getters)]
#[serde(rename_all = "camelCase")]
pub struct Schema {
    /// Fonto schema compiler version number
    #[builder(default)]
    version: FontoSchemaCompilerVersion,

    /// simple type definitions
    #[builder(default)]
    simple_types: Vec<fonto::SimpleType>,

    /// attribute definitions
    #[builder(default)]
    attributes: Vec<fonto::Attribute>,

    /// complex type definitions
    #[builder(default)]
    content_models: Vec<fonto::ContentModel>,

    /// element definitions
    #[builder(default)]
    elements: Vec<fonto::Element>,

    /// local element definitions.
    /// A "local element" specifically refers to an element declaration that is
    /// nested within a complex type definition.
    /// These elements are not globally available and can only be used within the context
    /// of the complex type where they are defined. They are useful for specifying elements
    /// that are relevant only within the context of a particular complex type and are not
    /// intended for reuse elsewhere in the schema.
    #[builder(default)]
    local_elements: Vec<fonto::LocalElement>,
}

impl Default for Schema {
    fn default() -> Self {
        Self {
            version: Default::default(),
            simple_types: vec![],
            attributes: vec![],
            content_models: vec![fonto::ContentModel::Empty {
                max_occurs: Some(1.into()),
                min_occurs: Some(1.into()),
            }],
            elements: vec![],
            local_elements: vec![],
        }
    }
}

impl Schema {
    pub fn assert_simpletype_idx(&self, count: usize) -> anyhow::Result<()> {
        assert!(
            self.simple_types.len() > count,
            "SimpleType index out of bounds"
        );
        Ok(())
    }

    pub fn assert_content_model_idx(&self, count: usize) -> anyhow::Result<()> {
        if self.content_models.len() > count {
            Ok(())
        } else {
            anyhow::bail!("ContentModel index out of bounds")
        }
    }

    pub fn assert_attribute_idx(&self, count: usize) -> anyhow::Result<()> {
        assert!(
            self.attributes.len() > count,
            "ContentModel index out of bounds"
        );
        Ok(())
    }

    pub fn assert_element_idx(&self, count: usize) -> anyhow::Result<()> {
        assert!(self.elements.len() > count, "Element index out of bounds");
        Ok(())
    }

    pub fn assert_local_element_idx(&self, count: usize) -> anyhow::Result<()> {
        assert!(
            self.local_elements.len() > count,
            "LocalElement index out of bounds"
        );
        Ok(())
    }

    pub fn get_content_model_empty_idx(&self) -> FontoDefinitionIdx {
        for (idx, cm) in self.content_models.iter().enumerate() {
            if cm.is_empty() {
                return idx;
            }
        }

        unreachable!("empty variant should have been added on initialization")
    }

    pub fn get_simple_type_idx(&self, st: &fonto::SimpleType) -> anyhow::Result<usize> {
        self.simple_types
            .iter()
            .position(|st1| st1 == st)
            .ok_or_else(|| anyhow::anyhow!("SimpleType not found"))
    }

    pub fn push_simple_type(&mut self, st: fonto::SimpleType) -> usize {
        st.validate_refs(self).unwrap();

        self.simple_types.push(st);
        self.simple_types.len() - 1
    }

    pub fn push_attribute(&mut self, attr: fonto::Attribute) -> usize {
        attr.validate_refs(self).unwrap();

        self.attributes.push(attr);
        self.attributes.len() - 1
    }

    pub fn push_element(&mut self, el: fonto::Element) -> usize {
        el.validate_refs(self).unwrap();

        self.elements.push(el);
        self.elements.len() - 1
    }

    pub fn push_local_element(&mut self, el: fonto::LocalElement) -> usize {
        el.validate_refs(self).unwrap();

        self.local_elements.push(el);
        self.local_elements.len() - 1
    }

    pub fn push_content_model(&mut self, cm: fonto::ContentModel) -> usize {
        cm.validate_refs(self).unwrap();

        self.content_models.push(cm);
        self.content_models.len() - 1
    }

    /// allocate a location for a content model so we can register the position
    /// in the exporter-tracked typehash map and prevent recursion
    /// this dummy will later on have to be replaced
    pub fn allocate_content_model(&mut self) -> usize {
        self.content_models.push(fonto::ContentModel::Empty {
            max_occurs: None,
            min_occurs: None,
        });
        self.content_models.len() - 1
    }

    pub fn set_content_model(&mut self, idx: usize, cm: fonto::ContentModel) {
        cm.validate_refs(self).unwrap();
        self.content_models[idx] = cm;
    }

    pub fn save_to_file(&self, path: impl AsRef<Path>) -> anyhow::Result<()> {
        Ok(std::fs::write(path, serde_json::to_string(self)?)?)
    }

    pub fn set_schema_version(&mut self, version: FontoSchemaCompilerVersion) {
        self.version = version;
    }
}

use derive_builder::Builder;
use derive_getters::Getters;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Builder)]
#[serde(rename_all = "camelCase")]
pub struct Attribute {
    #[serde(rename = "localName")]
    name: String,

    #[serde(rename = "namespaceURI")]
    #[builder(default)]
    namespace_uri: Option<String>,

    #[serde(rename = "use")]
    #[serde(serialize_with = "bool_to_string", deserialize_with = "string_to_bool")]
    #[builder(default)]
    required: bool,

    /// offset into the simple type definitions array
    simple_type_ref: usize,

    #[builder(default)]
    default_value: Option<String>,
}

impl Attribute {
    pub fn validate_refs(&self, schema: &super::Schema) -> anyhow::Result<()> {
        schema.assert_simpletype_idx(self.simple_type_ref)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Getters, Builder)]
#[serde(rename_all = "camelCase")]
pub struct AnyAttrConf {
    #[builder(default)]
    disallowed_namespace_names: Vec<String>,
    process_contents: AnyAttrValidation,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
#[serde(rename_all = "lowercase")]
pub enum AnyAttrValidation {
    Skip,
    Lax,
    Strict,
}

fn bool_to_string<S>(b: &bool, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    if *b {
        serializer.serialize_str("required")
    } else {
        serializer.serialize_str("optional")
    }
}

fn string_to_bool<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: &str = Deserialize::deserialize(deserializer)?;
    match s {
        "required" => Ok(true),
        "optional" => Ok(false),
        _ => Err(serde::de::Error::custom("Invalid value for boolean")),
    }
}

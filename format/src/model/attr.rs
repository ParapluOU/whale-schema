use crate::model;
use crate::model::simpletype::SimpleType;
use crate::model::{Comment, Ref};
use derive_builder::Builder;
use derive_getters::Getters;
use itertools::Itertools;
use std::cmp::Ordering;
use std::collections::{BTreeSet, HashMap};
use std::hash::{Hash, Hasher};
use std::ops::Deref;

#[derive(Debug, PartialEq, Eq, Default, Clone)]
pub struct Attributes(HashMap<String, Ref<Attribute>>);

impl Attributes {
    pub fn new(list: Vec<Ref<Attribute>>, schema: &model::Schema) -> Self {
        Self(
            list.into_iter()
                .map(|attr| {
                    (
                        schema
                            .get_attribute_name(&attr)
                            .expect("attr should have been defined")
                            .clone(),
                        attr,
                    )
                })
                .collect(),
        )
    }

    pub fn merge(mut self, other: Self) -> Self {
        self.0.extend(other.0.into_iter());
        self
    }

    pub fn get<'a>(&'a self, schema: &'a model::Schema) -> Vec<&'a Attribute> {
        self.0
            .values()
            .map(|attr| {
                schema
                    .get_attribute(attr)
                    .expect("attr should have been defined")
            })
            .collect()
    }

    pub fn as_vec(&self) -> Vec<&Ref<Attribute>> {
        self.0.values().sorted().collect::<Vec<_>>()
    }
}

impl Deref for Attributes {
    type Target = HashMap<String, Ref<model::Attribute>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Hash for Attributes {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.values().collect::<BTreeSet<_>>().hash(state)
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, Builder, Getters)]
pub struct Attribute {
    /// name of this attribute. may be duplicate with other attrs defined elsewhere
    pub name: String,

    // todo: support namespace
    /// whether the attribute is required
    required: bool,

    /// type definition reference in the schema
    pub typing: Ref<SimpleType>,

    /// comments associated with this attribute
    #[builder(default)]
    pub comments: Vec<Comment>,

    /// todo: create AST syntax to support this
    #[builder(default)]
    pub default_value: Option<String>,
}

impl Attribute {
    // pub fn name(&self) -> &str {
    //     self.name.as_str()
    // }
}

impl PartialOrd<Self> for Attribute {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.name.cmp(&other.name))
    }
}

impl Ord for Attribute {
    fn cmp(&self, other: &Self) -> Ordering {
        self.name.cmp(&other.name)
    }
}

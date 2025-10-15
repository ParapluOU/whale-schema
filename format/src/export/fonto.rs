use crate::export::Exporter;
use crate::formats::fonto;
use crate::formats::fonto::FontoSchemaCompilerVersion;
use crate::model;
use crate::model::{GetTypeHash, Group, GroupItem, GroupType, Schema};
use log::{debug, info};
use std::collections::HashMap;
use std::path::Path;

pub type FontoDefinitionIdx = usize;

#[derive(Default)]
pub struct FontoSchemaExporter {
    /// track all types we have already exported to the Fonto datastructure
    exported_type_ids: HashMap<model::TypeHash, FontoDefinitionIdx>,

    target_version: FontoSchemaCompilerVersion,

    result: fonto::Schema,
}

impl Exporter for FontoSchemaExporter {
    type Output = fonto::Schema;

    fn export_schema(mut self, schema: &model::Schema) -> anyhow::Result<Self::Output> {
        // go over all simpletypes and recursively resolve the dependencies
        info!("exporting Fonto SimpleTypes...");
        for st in schema.types_simple().values() {
            self.export_simple_type(st, schema)?;
        }

        // go over all elements to export the definitions
        info!("exporting Fonto elements...");
        for el in schema.elements().values() {
            self.export_element(el, schema)?;
        }

        // export attributes that reference simpletypes
        info!("exporting Fonto Attributes...");
        for attr in schema.types_attribute().values() {
            self.export_attribute(attr, schema)?;
        }

        // export remaining definitions in case there are definitions unused by elements,
        // perhaps of use for importing by other schemas
        info!("exporting Fonto ContentModels...");
        for gr in schema.types_group().values() {
            self.export_content_model(gr, schema)?;
        }

        self.result.set_schema_version(self.target_version.clone());

        Ok(self.result)
    }
}

impl FontoSchemaExporter {
    pub fn with_version(version: FontoSchemaCompilerVersion) -> Self {
        Self {
            exported_type_ids: Default::default(),
            target_version: version,
            result: Default::default(),
        }
    }

    pub fn export_to_file(
        mut self,
        schema: &model::Schema,
        path: impl AsRef<Path>,
    ) -> anyhow::Result<()> {
        let exported = self.export_schema(schema)?;
        exported.save_to_file(path)
    }

    fn create_content_model(
        &mut self,
        st: &Group,
        schema: &Schema,
    ) -> anyhow::Result<fonto::ContentModel> {
        debug!(
            "Creating ContentModel from Group definition #{}...",
            st.id()
        );

        let items = st
            .items()
            .iter()
            .map(|item| try {
                match item {
                    GroupItem::Element(el) => {
                        let el = el.resolve(schema);

                        // shoujld return existing position because it should have been exported already
                        let pos = self.export_element(el, schema)?;

                        fonto::ContentModel::LocalElement {
                            element_ref: pos,
                            max_occurs: el.max_occurs().map(Into::into),
                            min_occurs: Some(el.min_occurs().into()),
                        }
                    }
                    GroupItem::Group(gr) => {
                        self.create_content_model(gr.resolve(schema), schema)?
                    }
                }
            })
            .collect::<anyhow::Result<_>>()?;

        Ok(match st.ty() {
            GroupType::Sequence => fonto::ContentModel::Sequence {
                items,
                max_occurs: Some(1.into()),
                min_occurs: Some(1.into()),
            },
            GroupType::Choice => fonto::ContentModel::Choice {
                items,
                max_occurs: None,
                min_occurs: Some(0.into()),
            },
            GroupType::All => fonto::ContentModel::All { items },
        })
    }

    fn export_content_model(
        &mut self,
        st: &model::Group,
        schema: &model::Schema,
    ) -> anyhow::Result<FontoDefinitionIdx> {
        if self.exported_type_ids.contains_key(&st.id()) {
            return Ok(*self.exported_type_ids.get(&st.id()).unwrap());
        }

        debug!("Exporting Fonto ContentModel for Group #{}", st.id());

        let pos = self.result.allocate_content_model();

        // accounting to prevent double exporting
        self.exported_type_ids.insert(st.id(), pos);

        let cm = self.create_content_model(st, schema)?;

        self.result.set_content_model(pos, cm);

        Ok(pos)
    }

    fn export_element(
        &mut self,
        st: &model::Element,
        schema: &model::Schema,
    ) -> anyhow::Result<FontoDefinitionIdx> {
        if self.exported_type_ids.contains_key(&st.id()) {
            return Ok(*self.exported_type_ids.get(&st.id()).unwrap());
        }

        debug!("Exporting Fonto element #{}", st.name());

        // convert attribute definitions to their positions in the Fonto Scgema
        let attrs = st
            .group_merged_attributes(schema)
            .as_vec()
            .into_iter()
            .map(|attr| self.export_attribute(attr.resolve(schema), schema))
            .collect::<anyhow::Result<_>>()?;

        let mut builder = fonto::ElementBuilder::default();

        builder
            .name(st.name().clone())
            .attribute_refs(attrs)
            // todo: support namespaces
            // .namespace_uri(todo!())
            .is_mixed(st.is_mixed_content(schema))
            .min_occurs(Some(st.min_occurs().into()))
            .max_occurs(st.max_occurs().map(Into::into));

        match st.typing() {
            // might be recursively added new
            model::TypeRef::Group(gr) => {
                builder.content_model_ref(self.export_content_model(gr.resolve(schema), schema)?);
            }
            // should already exist
            model::TypeRef::Simple(sty) => {
                // the content model for the element is validated by the SimpleType
                builder
                    .simple_type_ref(self.export_simple_type(sty.resolve(schema), schema)?.into());

                // fonto will throw errors without this
                builder.is_mixed(true);

                // see format/src/tests/fonto/date/date-test.json
                // for an example to see that empty elements in terms of having children
                // have a content model ref defined in addition to the simpleTypeRef
                builder.content_model_ref(self.result.get_content_model_empty_idx());
            }
        };

        let fonto_element = builder.build()?;

        let pos = if st.is_local(schema) {
            self.result.push_local_element(fonto_element)
        } else {
            self.result.push_element(fonto_element)
        };

        self.exported_type_ids.insert(st.id(), pos);

        Ok(pos)
    }

    fn export_attribute(
        &mut self,
        st: &model::Attribute,
        schema: &model::Schema,
    ) -> anyhow::Result<FontoDefinitionIdx> {
        if self.exported_type_ids.contains_key(&st.id()) {
            return Ok(*self.exported_type_ids.get(&st.id()).unwrap());
        }

        debug!("Exporting Fonto attribute #{}", st.name());

        let typeref = self.export_simple_type(st.typing().resolve(schema), schema)?;

        let attr_idx = self.result.push_attribute(
            fonto::AttributeBuilder::default()
                .name(st.name().clone())
                // todo: namespace support
                // .namespace_uri(st.namespace_uri().cloned())
                .required(*st.required())
                .default_value(st.default_value().clone())
                .simple_type_ref(typeref)
                .build()?,
        );

        self.exported_type_ids.insert(st.id(), attr_idx);

        Ok(attr_idx)
    }

    /// export simple type into the fonto schema and return its position
    fn export_simple_type(
        &mut self,
        st: &model::SimpleType,
        schema: &model::Schema,
    ) -> anyhow::Result<FontoDefinitionIdx> {
        if self.exported_type_ids.contains_key(&st.id()) {
            return Ok(*self.exported_type_ids.get(&st.id()).unwrap());
        }

        debug!("Exporting Fonto SimpleType #{}", st.id());

        let res = match st {
            model::SimpleType::Derived { base, restrictions } => {
                let base = self.export_simple_type(base.resolve(schema), schema)?;

                self.result.push_simple_type(fonto::SimpleType::Derived {
                    base,
                    restrictions: restrictions.clone().into(),
                })
            }
            model::SimpleType::Union { member_types } => {
                let mut exported_members = vec![];

                for i in 0..member_types.len() {
                    let member =
                        self.export_simple_type(member_types[i].resolve(schema), schema)?;
                    exported_members.push(member);
                }

                self.result.push_simple_type(fonto::SimpleType::Union {
                    member_types: exported_members,
                })
            }
            model::SimpleType::List {
                item_type,
                separator,
            } => {
                let exported_item = self.export_simple_type(item_type.resolve(schema), schema)?;

                self.result.push_simple_type(fonto::SimpleType::List {
                    item_type: exported_item,
                    separator: separator.clone(),
                })
            }
            model::SimpleType::Builtin { name } => self
                .result
                .push_simple_type(fonto::SimpleType::Builtin { name: name.into() }),
        };

        // accounting to prevent double exporting
        self.exported_type_ids.insert(st.id(), res);

        Ok(res)
    }
}

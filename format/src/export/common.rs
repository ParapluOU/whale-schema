use crate::model;

pub trait Exporter {
    type Output;

    fn export_schema(self, schema: &model::Schema) -> anyhow::Result<Self::Output>;
}

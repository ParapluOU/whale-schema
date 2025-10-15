use crate::model;

pub trait Importer {
    fn import_schema(&mut self) -> anyhow::Result<model::Schema>;
}

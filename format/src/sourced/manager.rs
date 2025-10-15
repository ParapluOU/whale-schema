use crate::ast::SchemaFile;
use crate::sourced::SourcedSchemaFile;
use derive_getters::Getters;
use std::collections::HashMap;
use std::ops::Deref;
use std::path;
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[derive(Debug)]
pub struct SchemaFileManager {
    /// directory where the entry schema file is located
    root: PathBuf,

    /// collection of all schema files that have been loaded
    map: HashMap<PathBuf, Arc<SchemaFile>>,
}

impl SchemaFileManager {
    /// for use in tests and stubs
    pub fn new() -> Self {
        Self {
            root: PathBuf::new(),
            map: HashMap::new(),
        }
    }

    pub fn from_root_schema(path: impl AsRef<Path>) -> anyhow::Result<SourcedSchemaFile> {
        let root = path
            .as_ref()
            .parent()
            .ok_or(anyhow::anyhow!("parent dir of entry schema not found"))?
            .to_path_buf();

        let mut man = Self {
            root,
            map: HashMap::new(),
        };

        let schema = man.add_schema_file_path(&path)?;

        let singled_manager = Arc::new(man);

        Ok(SourcedSchemaFile {
            schema,
            path: path.as_ref().to_path_buf(),
            manager: singled_manager,
        })
    }

    pub fn add_schema_file_path(
        &mut self,
        path: impl AsRef<Path>,
    ) -> anyhow::Result<Arc<SchemaFile>> {
        let path = path::absolute(path.as_ref())?;

        // parent dir of the schema file
        let schema_dir = path
            .parent()
            .ok_or(anyhow::anyhow!("schema dir not found"))?
            .to_path_buf();

        if self.map.contains_key(&path) {
            return Ok(self.map.get(&path).unwrap().clone());
        }

        // register schema so it can be resolved to
        self.map
            .insert(path.clone(), Arc::new(SchemaFile::new_file(&path)?));

        // recursively go through the imports
        let schema = self.map.get(&path).unwrap().clone();
        for import in &schema.imports {
            // absolute path of the target schema that we want to import
            let import_abspath = import.absolute_path(&schema_dir);

            // add it to the manager but ignore if its known
            self.add_schema_file_path(import_abspath)?;
        }

        Ok(self.map.get(&path).unwrap().clone())
    }

    pub fn types_count(&self) -> usize {
        self.map.values().map(|schema| schema.types_count()).sum()
    }
}

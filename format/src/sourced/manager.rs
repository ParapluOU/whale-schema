use crate::ast::SchemaFile;
use crate::sourced::SourcedSchemaFile;
use anyhow::Context;
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

        // If already loaded, return cached version (prevents infinite recursion)
        if self.map.contains_key(&path) {
            return Ok(self.map.get(&path).unwrap().clone());
        }

        // Parse the file WITHOUT validating imports (to avoid recursion issues)
        // We resolve the file path first (handles .whas extension)
        let resolved_path = SchemaFile::resolve_file_path(&path)?;
        let content = std::fs::read_to_string(&resolved_path)
            .context(format!("reading schema from {}", resolved_path.display()))?;
        let schema = SchemaFile::parse(&content)
            .context(format!("parsing schema from {}", resolved_path.display()))?;

        // Add to cache IMMEDIATELY before processing imports
        // This enables cycle detection - if an import references this file again,
        // the contains_key check above will catch it
        let schema_arc = Arc::new(schema);
        self.map.insert(path.clone(), schema_arc.clone());

        // NOW recursively process imports (cycle detection works!)
        let schema_ref = self.map.get(&path).unwrap().clone();
        for import in &schema_ref.imports {
            // absolute path of the target schema that we want to import
            let import_abspath = import.absolute_path(&schema_dir);

            // add it to the manager (will use cache if already loaded)
            self.add_schema_file_path(import_abspath)?;
        }

        Ok(schema_arc)
    }

    pub fn types_count(&self) -> usize {
        self.map.values().map(|schema| schema.types_count()).sum()
    }
}

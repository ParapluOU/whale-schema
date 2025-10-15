use crate::ast;
use crate::ast::{SchemaFile, TypeDef};
use crate::sourced::SchemaFileManager;
use derive_getters::Getters;
use std::collections::HashMap;
use std::ops::Deref;
use std::path;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// AST SchemaFile that has been annotated with the path from which it was loaded
/// and the ability to retrieve stuff from other schemas
#[derive(Clone, Debug, Getters)]
pub struct SourcedSchemaFile {
    /// the schema file that was loaded
    pub schema: Arc<SchemaFile>,

    /// the path from which the schema was loaded
    pub path: PathBuf,

    /// the manager that loaded the schema
    pub manager: Arc<SchemaFileManager>,
}

impl SourcedSchemaFile {
    pub fn from_ast_schema(schema: SchemaFile) -> Self {
        Self {
            schema: Arc::new(schema),
            path: Default::default(),
            manager: Arc::new(SchemaFileManager::new()),
        }
    }

    // resolve across imports
    pub fn types(&self) -> Vec<&TypeDef> {
        let own_types = self.schema.types_own();

        if self.schema.has_imports() {
            todo!()
        }

        own_types
    }
}

impl From<ast::SchemaFile> for SourcedSchemaFile {
    fn from(schema: ast::SchemaFile) -> Self {
        Self::from_ast_schema(schema)
    }
}

impl Deref for SourcedSchemaFile {
    type Target = Arc<SchemaFile>;

    fn deref(&self) -> &Arc<SchemaFile> {
        &self.schema
    }
}

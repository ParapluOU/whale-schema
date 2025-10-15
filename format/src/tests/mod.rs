use crate::sourced::{SchemaFileManager, SourcedSchemaFile};
use crate::*;

mod ast;
mod compiler;
mod fonto;
mod grammar;
mod imports;
mod types;

pub fn get_test_schema_ast() -> SourcedSchemaFile {
    // crate::ast::SchemaFile::new_file("./test.whas").unwrap()
    SchemaFileManager::from_root_schema("./test.whas").unwrap()
}

pub fn get_compiled_schema() -> model::Schema {
    let source = get_test_schema_ast();
    crate::compiler::compile(&source).unwrap()
}

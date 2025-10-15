use crate::sourced::SchemaFileManager;
use crate::{ast, model};
use anyhow::Context;
use std::path::PathBuf;
use std::{env, fs, path};

/// test whether imports can be validated, meaning parsed for well-formedness
/// and found using filepaths
#[test]
fn test_import_resolution_simple() {
    let content = fs::read_to_string("./src/tests/schemas/imports/simple.whas").unwrap();
    let schema = ast::SchemaFile::parse(&content).unwrap();
    let ctxdir = env::current_dir()
        .unwrap()
        .join("src/tests/schemas/imports/");

    schema.validate_imports(ctxdir).unwrap();
}

#[test]
fn test_import_resolution_glob() {
    let content = fs::read_to_string("./src/tests/schemas/imports.whas").unwrap();
    let schema = ast::SchemaFile::parse(&content).unwrap();
    let ctxdir = env::current_dir().unwrap().join("src/tests/schemas/");

    schema.validate_imports(ctxdir).unwrap();
}

/// test an example import
#[test]
fn test_import_simple() {
    // load referencing schema through schema file manager
    let ctxdir = env::current_dir()
        .unwrap()
        .join("src/tests/schemas/imports");

    let referencing_schema =
        SchemaFileManager::from_root_schema("./src/tests/schemas/imports/simple.whas").unwrap();

    // load bare target schema as reference
    let target_schema = ast::SchemaFile::from_file("./src/tests/schemas/aliasing.whas").unwrap();
    let target_types = target_schema.types_simple();

    // same number of types should be known in both
    assert_eq!(
        referencing_schema.types_count(),
        target_schema.types_count()
    );
}

/// test cyclic imports (A imports B, B imports A)
#[test]
fn test_import_cyclic() {
    let result = SchemaFileManager::from_root_schema("./src/tests/schemas/imports/cycle-a.whas");

    // This should not panic or cause infinite recursion
    // Both schemas should be loaded successfully
    assert!(result.is_ok(), "Cyclic imports should be handled gracefully");

    let schema = result.unwrap();

    // Should have types from both files
    assert_eq!(schema.types_count(), 2, "Should load types from both cyclic schemas");
}

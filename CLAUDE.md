# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

WHAS (Whale Schema Definition) is an XML schema language designed to simplify XML schema creation. It compiles to Fonto compiled schemas (JSON) and eventually XSD. The goal is to make XML schema development accessible without requiring specialized GUI tools.

## Build & Development Commands

### Building and Running
```bash
# Build the project (in format/ directory)
cd format && cargo build

# Run the compiler
cd format && cargo run -- <input.whas> [OPTIONS]

# Build release binary
cd format && cargo build --release
```

### Testing
```bash
# Run all tests (may take time, use larger timeout if needed)
cd format && cargo test

# Run specific test module
cd format && cargo test ast
cd format && cargo test compiler
cd format && cargo test fonto

# Run tests with output
cd format && cargo test -- --nocapture
```

### Compilation Examples
```bash
# Compile to Fonto schema (default)
cd format && cargo run -- ../schema.whas -o ./output

# Specify Fonto version
cd format && cargo run -- ../schema.whas --fonto-version "8.8" -o ./output
```

### Code Quality
```bash
# Check for errors (use timeout of up to 10m if needed)
cd format && cargo check

# Add new dependencies
cd format && cargo add <crate-name>
```

## Architecture

### Project Structure

The codebase is organized as a Rust workspace with the main compiler in `format/`:

- **AST Layer** (`format/src/ast/`): Parses `.whas` files using Pest parser into Abstract Syntax Tree
  - Grammar defined in `format/schema.pest`
  - Modules: `elements.rs`, `types.rs`, `blocks.rs`, `attrs.rs`, `imports.rs`, etc.

- **Model Layer** (`format/src/model/`): Internal representation of schemas
  - `schema.rs`: Main Schema struct that holds all definitions
  - `element.rs`, `type.rs`, `group.rs`: Core schema components
  - `primitive.rs`, `simpletype.rs`: Type system primitives
  - Uses builders pattern extensively (via `derive_builder`)

- **Compiler** (`format/src/compiler/`): Transforms AST → Model
  - Entry point: `compile()` in `compiler/mod.rs`
  - Two-phase compilation:
    1. `compile_type_definitions()`: Register all type definitions with IDs
    2. `compile_elements()`: Compile element definitions
  - Handles recursive type resolution via `SchemaObjId` references

- **Export Layer** (`format/src/export/`, `format/src/formats/`):
  - `export/fonto.rs`: Main Fonto exporter
  - `formats/fonto/`: Fonto-specific serialization logic
    - `schema.rs`, `element.rs`, `content_model.rs`, `attribute.rs`
    - Version handling in `version.rs`

- **Import System** (`format/src/sourced/`):
  - `manager.rs`: SchemaFileManager handles multi-file schemas
  - `file.rs`: SourcedSchemaFile wraps AST with source tracking
  - Supports glob patterns for imports

- **CLI** (`format/src/cli/`): Command-line interface using clap

### Key Concepts

**Type System**: WHAS supports primitives (String, Int, Date, etc.), custom types, and groups. Types can be:
- **Block Types**: Define structure with nested elements (like XSD complexType)
- **Simple Types**: Primitives or regex restrictions (like XSD simpleType)
- **Aliases**: Type references that resolve to other types

**Splatting**: The `...TypeName` syntax allows inline expansion of type definitions (like XSD groups).

**Modifiers**:
- Duplicity: `?` (optional), `*` (zero-or-more), `+` (one-or-more), `[n..m]` (range)
- Occurrence: `?` (choice), `!` (all), default is sequence
- Mixed content: `x` prefix allows text nodes

**Import Resolution**: Schemas can import from other `.whas` files. The SchemaFileManager handles:
- Glob pattern expansion
- Cyclic import detection (partial support)
- Cross-file type resolution

## Rust Language Features

This project uses nightly Rust features:
- `associated_type_bounds`
- `let_chains`
- `try_blocks`
- `absolute_path`

Ensure you're using Rust nightly when building.

## Testing Conventions

Test files are in `format/src/tests/`. The test suite uses a reference schema at `./test.whas`.

Helper functions in `tests/mod.rs`:
- `get_test_schema_ast()`: Returns parsed AST
- `get_compiled_schema()`: Returns fully compiled schema

## Common Workflows

### Adding a new WHAS language feature:
1. Update `format/schema.pest` with grammar rules
2. Add AST types in `format/src/ast/`
3. Update compiler in `format/src/compiler/mod.rs`
4. Add model representation in `format/src/model/`
5. Update exporters in `format/src/formats/`
6. Add tests in `format/src/tests/`

### Debugging compilation issues:
- Enable logging (already configured in `main.rs` via simplelog)
- Check `debug.log` file for detailed compilation traces
- Use `cargo check` frequently during development

## Output Formats

### Fonto Schema JSON
Generated schemas are versioned for compatibility with specific Fonto versions. The `--fonto-version` flag controls the output schema format version.

### XSD (Planned)
XSD export is not yet implemented (see `main.rs:62`).

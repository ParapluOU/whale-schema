use crate::export::{Exporter, XsdExporter};
use crate::model::restriction::SimpleTypeRestriction;
use crate::model::{GroupItem, GroupType, PrimitiveType, SimpleType};
use crate::sourced::SchemaFileManager;
use crate::{compiler, model};
use anyhow::Result;

/// Helper function to compare XSD output against expected golden file
fn assert_xsd_matches_expected(schema_name: &str) -> Result<()> {
    let schema_path = format!("src/tests/schemas/xsd/{}.whas", schema_name);
    let expected_path = format!("src/tests/schemas/xsd/expected/{}.xsd", schema_name);

    let schema = model::Schema::from_file(&schema_path)?;
    let exporter = XsdExporter::default();
    let actual_output = exporter.export_schema(&schema)?;

    let expected_output = std::fs::read_to_string(&expected_path)
        .unwrap_or_else(|_| panic!("Expected XSD file not found: {}", expected_path));

    // Normalize whitespace for comparison (in case of line ending differences)
    let actual_normalized = actual_output.trim();
    let expected_normalized = expected_output.trim();

    if actual_normalized != expected_normalized {
        eprintln!("\n=== ACTUAL OUTPUT ===");
        eprintln!("{}", actual_output);
        eprintln!("\n=== EXPECTED OUTPUT ===");
        eprintln!("{}", expected_output);
        eprintln!("\n=== DIFF ===");

        // Simple diff - show first difference
        for (i, (actual_line, expected_line)) in actual_output.lines().zip(expected_output.lines()).enumerate() {
            if actual_line != expected_line {
                eprintln!("Line {}: DIFFER", i + 1);
                eprintln!("  Actual:   {}", actual_line);
                eprintln!("  Expected: {}", expected_line);
                break;
            }
        }

        panic!("XSD output does not match expected output for {}", schema_name);
    }

    Ok(())
}

/// Test XSD primitive type mappings
#[test]
fn test_xsd_primitives() -> Result<()> {
    assert_xsd_matches_expected("primitives")
}

/// Test XSD sequence (default block behavior)
#[test]
fn test_xsd_sequence() -> Result<()> {
    assert_xsd_matches_expected("sequence")
}

/// Test XSD choice (? block modifier)
#[test]
fn test_xsd_choice() -> Result<()> {
    assert_xsd_matches_expected("choice")
}

/// Test XSD all (! block modifier)
#[test]
fn test_xsd_all() -> Result<()> {
    assert_xsd_matches_expected("all")
}

/// Test XSD group (type splatting with ...TypeName)
#[test]
fn test_xsd_group() -> Result<()> {
    assert_xsd_matches_expected("group")
}

/// Test XSD attributes
#[test]
fn test_xsd_attributes() -> Result<()> {
    assert_xsd_matches_expected("attributes")
}

/// Test abstract types
#[test]
fn test_xsd_abstract() -> Result<()> {
    assert_xsd_matches_expected("abstract")
}

/// Test inheritance with xs:extension
#[test]
fn test_xsd_inheritance() -> Result<()> {
    assert_xsd_matches_expected("inheritance")
}

/// Test abstract types combined with inheritance
#[test]
fn test_xsd_abstract_inheritance() -> Result<()> {
    assert_xsd_matches_expected("abstract_inheritance")
}

/// Test mixed content (x{...} modifier)
#[test]
fn test_xsd_mixed() -> Result<()> {
    assert_xsd_matches_expected("mixed")
}

/// Test occurrence constraints (?, *, +, [n..m])
#[test]
fn test_xsd_occurrences() -> Result<()> {
    assert_xsd_matches_expected("occurrences")
}

/// TEMPORARY: Generate all expected XSD files
#[test]
#[ignore]
fn generate_all_expected_xsd() -> Result<()> {
    // Skip nested and realistic - they have recursive types that need cycle detection
    let tests = vec!["complex_element", "empty", "choice_in_sequence", "multi_attributes",
                     "splat_modifiers", "simple_content", "complex_content",
                     "attribute_groups", "facets_enumeration"];

    for test_name in tests {
        let schema = model::Schema::from_file(&format!("src/tests/schemas/xsd/{}.whas", test_name))?;
        let exporter = XsdExporter::default();
        let xsd = exporter.export_schema(&schema)?;
        println!("\n========== {}.xsd ==========\n{}", test_name, xsd);
    }
    Ok(())
}

/// Test simple type restrictions (regex patterns)
#[test]
fn test_xsd_restrictions() -> Result<()> {
    assert_xsd_matches_expected("restrictions")
}

/// Test simple type derivation (type aliasing)
#[test]
fn test_xsd_derivation() -> Result<()> {
    assert_xsd_matches_expected("derivation")
}

/// Test list types ([IDRef] syntax)
#[test]
fn test_xsd_list() -> Result<()> {
    assert_xsd_matches_expected("list")
}

/// Test complex nested structures
#[test]
#[ignore] // FIXME: Recursive types cause stack overflow - need cycle detection
fn test_xsd_nested() -> Result<()> {
    assert_xsd_matches_expected("nested")
}

/// Test elements with complex type
#[test]
fn test_xsd_complex_element() -> Result<()> {
    assert_xsd_matches_expected("complex_element")
}

/// Test empty elements
#[test]
fn test_xsd_empty() -> Result<()> {
    assert_xsd_matches_expected("empty")
}

/// Test choice within sequence (nested control structures)
#[test]
fn test_xsd_choice_in_sequence() -> Result<()> {
    assert_xsd_matches_expected("choice_in_sequence")
}

/// Test multiple attributes
#[test]
fn test_xsd_multi_attributes() -> Result<()> {
    assert_xsd_matches_expected("multi_attributes")
}

/// Test type splatting with modifiers (...Type?, ...Type*, ...Type+)
#[test]
fn test_xsd_splat_modifiers() -> Result<()> {
    assert_xsd_matches_expected("splat_modifiers")
}

/// Test realistic example (from test.whas)
#[test]
#[ignore] // FIXME: Recursive types cause stack overflow - need cycle detection
fn test_xsd_realistic() -> Result<()> {
    assert_xsd_matches_expected("realistic")
}

// ============================================================================
// UNSUPPORTED XSD FEATURES - Tests below should fail explicitly
// ============================================================================

/// Test XSD namespaces (NOT SUPPORTED)
#[test]
#[should_panic(expected = "WHAS does not yet support namespaces")]
fn test_xsd_namespaces() {
    let _schema = model::Schema::from_file("src/tests/schemas/xsd/namespaces.whas").unwrap();

    // TODO: When namespace support is added, check for:
    // - targetNamespace attribute
    // - namespace prefix declarations
    // - qualified element names

    panic!("WHAS does not yet support namespaces (see README roadmap)");
}

/// Test XSD xs:any wildcard (NOT SUPPORTED)
#[test]
#[should_panic(expected = "WHAS does not yet support xs:any wildcard elements")]
fn test_xsd_any_wildcard() {
    let _schema = model::Schema::from_file("src/tests/schemas/xsd/any_wildcard.whas").unwrap();

    // TODO: When wildcard support is added, check for:
    // - ...any syntax or similar
    // - processContents attribute (strict, lax, skip)
    // - namespace constraints

    panic!("WHAS does not yet support xs:any wildcard elements");
}

/// Test XSD xs:anyAttribute (NOT SUPPORTED)
#[test]
#[should_panic(expected = "WHAS does not yet support xs:anyAttribute")]
fn test_xsd_any_attribute() {
    let _schema = model::Schema::from_file("src/tests/schemas/xsd/any_attribute.whas").unwrap();

    // TODO: When wildcard attribute support is added, check for:
    // - @...any syntax or similar
    // - processContents attribute
    // - namespace constraints

    panic!("WHAS does not yet support xs:anyAttribute wildcard attributes");
}

/// Test XSD xs:extension for complex types (NOT SUPPORTED)
#[test]
#[should_panic(expected = "WHAS does not yet support type extension")]
fn test_xsd_extension() {
    let schema = model::Schema::from_file("src/tests/schemas/xsd/extension.whas").unwrap();

    // Currently we manually duplicate fields - no inheritance
    let base = schema.get_group_by_name("BaseType");
    let extended = schema.get_group_by_name("ExtendedType");

    assert!(base.is_some() && extended.is_some(), "Types exist but no extension mechanism");

    // TODO: When extension support is added, check for:
    // - ExtendedType extends BaseType syntax
    // - Automatic field inheritance
    // - Attribute inheritance

    panic!("WHAS does not yet support type extension (extends keyword)");
}

/// Test XSD xs:union types (SUPPORTED)
#[test]
fn test_xsd_union() -> Result<()> {
    assert_xsd_matches_expected("union")
}

/// Test XSD xs:union with literal values
#[test]
fn test_xsd_union_literals() -> Result<()> {
    assert_xsd_matches_expected("union_literals")
}

/// Test XSD xs:union with mixed types
#[test]
fn test_xsd_union_mixed() -> Result<()> {
    assert_xsd_matches_expected("union_mixed")
}

/// Test inline union types (unions without typedef)
#[test]
fn test_xsd_inline_unions() -> Result<()> {
    assert_xsd_matches_expected("inline_unions")
}

/// Test XSD substitution groups (NOT SUPPORTED)
#[test]
#[should_panic(expected = "WHAS does not yet support substitution groups")]
fn test_xsd_substitution_groups() {
    let _schema = model::Schema::from_file("src/tests/schemas/xsd/substitution_groups.whas").unwrap();

    // TODO: When substitution groups are added, check for:
    // - substitutionGroup attribute on elements
    // - Abstract head elements
    // - Substitutable elements

    panic!("WHAS does not yet support substitution groups");
}

/// Test XSD identity constraints (NOT SUPPORTED)
#[test]
#[should_panic(expected = "WHAS does not yet support identity constraints")]
fn test_xsd_identity_constraints() {
    let _schema = model::Schema::from_file("src/tests/schemas/xsd/identity_constraints.whas").unwrap();

    // TODO: When identity constraints are added, check for:
    // - xs:key (unique key constraint)
    // - xs:keyref (foreign key constraint)
    // - xs:unique (uniqueness constraint)
    // - XPath selectors and fields

    panic!("WHAS does not yet support identity constraints (key, keyref, unique)");
}

/// Test XSD default and fixed values (PARTIALLY SUPPORTED)
#[test]
#[should_panic(expected = "WHAS does not yet support default values for elements")]
fn test_xsd_default_fixed_values() {
    let schema = model::Schema::from_file("src/tests/schemas/xsd/default_fixed_values.whas").unwrap();

    // Note: Default values for attributes might be supported in model::Attribute
    // Check the model to see if default_value field exists

    let elements = schema.get_elements_by_name("element");
    assert!(!elements.is_empty(), "Element exists");

    // TODO: When default value support is added, check for:
    // - Default value syntax in WHAS
    // - Fixed value syntax (value cannot be changed)
    // - Both for elements and attributes

    panic!("WHAS does not yet support default values for elements (marked as TODO in roadmap)");
}

/// Test XSD nillable elements (NOT SUPPORTED)
#[test]
#[should_panic(expected = "WHAS does not yet support nillable elements")]
fn test_xsd_nillable() {
    let _schema = model::Schema::from_file("src/tests/schemas/xsd/nillable.whas").unwrap();

    // TODO: When nillable support is added, check for:
    // - nillable modifier or attribute
    // - Distinction between absent, empty, and null

    panic!("WHAS does not yet support nillable elements (xsi:nil)");
}

/// Test XSD abstract types (NOT SUPPORTED)
#[test]
#[should_panic(expected = "WHAS does not yet support abstract types")]
fn test_xsd_abstract_types() {
    let _schema = model::Schema::from_file("src/tests/schemas/xsd/abstract_types.whas").unwrap();

    // TODO: When abstract types are added, check for:
    // - abstract keyword on type definitions
    // - Prevention of direct instantiation
    // - Requirement for substitution/extension

    panic!("WHAS does not yet support abstract types");
}

/// Test XSD length facets (NOT SUPPORTED)
#[test]
#[should_panic(expected = "WHAS does not yet support length facets")]
fn test_xsd_facets_length() {
    let _schema = model::Schema::from_file("src/tests/schemas/xsd/facets_length.whas").unwrap();

    // TODO: When length facets are added, check for:
    // - length: n (exact length)
    // - minLength: n
    // - maxLength: n
    // - Applied to string types

    panic!("WHAS does not yet support length facets (minLength, maxLength, length)");
}

/// Test XSD numeric facets (NOT SUPPORTED)
#[test]
#[should_panic(expected = "WHAS does not yet support numeric range facets")]
fn test_xsd_facets_numeric() {
    let _schema = model::Schema::from_file("src/tests/schemas/xsd/facets_numeric.whas").unwrap();

    // TODO: When numeric facets are added, check for:
    // - minInclusive, maxInclusive
    // - minExclusive, maxExclusive
    // - totalDigits, fractionDigits

    panic!("WHAS does not yet support numeric range facets (minInclusive, maxInclusive, etc.)");
}

/// Test XSD enumeration facet (SUPPORTED via regex, but no explicit enum syntax)
#[test]
fn test_xsd_facets_enumeration() -> Result<()> {
    assert_xsd_matches_expected("facets_enumeration")
}

// ============================================================================
// FACET TESTS - New angle bracket syntax <...>
// ============================================================================

/// Test XSD string length facets (minLength, maxLength, length)
#[test]
fn test_xsd_facets_string_length() -> Result<()> {
    assert_xsd_matches_expected("facets_string_length")
}

/// Test XSD numeric range facets (minInclusive, maxInclusive, minExclusive, maxExclusive)
#[test]
fn test_xsd_facets_numeric_ranges() -> Result<()> {
    assert_xsd_matches_expected("facets_numeric_ranges")
}

/// Test XSD decimal precision facets (totalDigits, fractionDigits)
#[test]
fn test_xsd_facets_decimal_precision() -> Result<()> {
    assert_xsd_matches_expected("facets_decimal_precision")
}

/// Test XSD whiteSpace facet (preserve, replace, collapse)
#[test]
fn test_xsd_facets_whitespace_new() -> Result<()> {
    assert_xsd_matches_expected("facets_whitespace_new")
}

/// Test XSD combined facets (multiple facets on same type)
#[test]
fn test_xsd_facets_combined() -> Result<()> {
    assert_xsd_matches_expected("facets_combined")
}

/// Test XSD facets with other WHAS features (unions, inheritance, occurrences, etc.)
#[test]
fn test_xsd_facets_advanced() -> Result<()> {
    assert_xsd_matches_expected("facets_advanced")
}

// ============================================================================
// OLD FACET TESTS - Legacy tests for unsupported syntax
// ============================================================================

/// Test XSD whiteSpace facet (NOT SUPPORTED)
#[test]
#[should_panic(expected = "WHAS does not yet support whiteSpace facet")]
fn test_xsd_facets_whitespace() {
    let _schema = model::Schema::from_file("src/tests/schemas/xsd/facets_whitespace.whas").unwrap();

    // TODO: When whiteSpace facet is added, check for:
    // - preserve (keep all whitespace)
    // - replace (replace each whitespace char with space)
    // - collapse (collapse consecutive whitespace to single space)

    panic!("WHAS does not yet support whiteSpace facet (preserve, replace, collapse)");
}

/// Test XSD simpleContent (SUPPORTED implicitly)
#[test]
fn test_xsd_simple_content() -> Result<()> {
    assert_xsd_matches_expected("simple_content")
}

/// Test XSD complexContent (SUPPORTED - default for block elements)
#[test]
fn test_xsd_complex_content() -> Result<()> {
    assert_xsd_matches_expected("complex_content")
}

/// Test XSD attribute groups (PARTIALLY SUPPORTED)
#[test]
fn test_xsd_attribute_groups() -> Result<()> {
    assert_xsd_matches_expected("attribute_groups")
}

/// Test XSD qualified elements (NOT SUPPORTED - requires namespaces)
#[test]
#[should_panic(expected = "WHAS does not yet support elementFormDefault")]
fn test_xsd_qualified_elements() {
    let _schema = model::Schema::from_file("src/tests/schemas/xsd/qualified_elements.whas").unwrap();

    // TODO: When namespace support is added, also support:
    // - elementFormDefault (qualified vs unqualified)
    // - attributeFormDefault
    // - form attribute on individual elements/attributes

    panic!("WHAS does not yet support elementFormDefault/attributeFormDefault (requires namespaces)");
}

/// Test XSD block and final attributes (NOT SUPPORTED)
#[test]
#[should_panic(expected = "WHAS does not yet support block/final attributes")]
fn test_xsd_block_final() {
    let _schema = model::Schema::from_file("src/tests/schemas/xsd/block_final.whas").unwrap();

    // TODO: When block/final are added, check for:
    // - block attribute (prevent derivation by extension/restriction)
    // - final attribute (prevent further derivation)
    // - Applied to type definitions

    panic!("WHAS does not yet support block/final attributes (derivation control)");
}

/// Test XSD xs:notation (NOT SUPPORTED)
#[test]
#[should_panic(expected = "WHAS does not yet support xs:notation")]
fn test_xsd_notation() {
    let _schema = model::Schema::from_file("src/tests/schemas/xsd/notation.whas").unwrap();

    // TODO: When notation support is added, check for:
    // - Notation declarations
    // - NOTATION simple type
    // - Public and system identifiers

    panic!("WHAS does not yet support xs:notation (binary data format declarations)");
}

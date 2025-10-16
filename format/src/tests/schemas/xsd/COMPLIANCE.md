# WHAS XSD Compliance Matrix

This document shows which XSD features are supported by WHAS and which are not yet implemented.

## ✅ Fully Supported (21 features)

| XSD Feature | WHAS Syntax | Test File | Notes |
|-------------|-------------|-----------|-------|
| Primitive types | `String`, `Int`, `Bool`, `Date`, etc. | `primitives.whas` | All XSD built-in types mapped |
| xs:sequence | `{ ... }` (default) | `sequence.whas` | Default block behavior |
| xs:choice | `Type: ?{ ... }` with splat | `choice.whas` | Via type with choice modifier |
| xs:all | `!{ ... }` | `all.whas` | Exclamation prefix |
| xs:group | `...TypeName` | `group.whas` | Type splatting |
| Attributes | `@name: Type` | `attributes.whas` | Required and optional |
| Mixed content | `x{ ... }` | `mixed.whas` | Mixed content modifier |
| Occurrence constraints | `?`, `*`, `+`, `[n..m]` | `occurrences.whas` | minOccurs/maxOccurs |
| Pattern restrictions | `/regex/` | `restrictions.whas` | xs:pattern facet |
| Type derivation | `Type: BaseType` | `derivation.whas` | Type aliasing |
| List types | `[IDRef]` | `list.whas` | xs:list |
| Recursive types | Nested type references | `nested.whas` | Self-referencing types |
| Complex elements | Element with children + attrs | `complex_element.whas` | xs:complexType |
| Empty elements | `{}` | `empty.whas` | Elements with no content |
| Nested control structures | Groups within sequences | `choice_in_sequence.whas` | Composition |
| Multiple attributes | Multiple `@` declarations | `multi_attributes.whas` | Any number of attributes |
| Type splatting | `...Type` in blocks | `splat_modifiers.whas` | Group reuse |
| Realistic schemas | Complex nested structures | `realistic.whas` | Real-world example |
| xs:simpleContent | Simple type + attributes | `simple_content.whas` | Element with text + attrs |
| xs:complexContent | Child elements | `complex_content.whas` | Default for blocks |
| Enumeration (via regex) | `/val1\|val2\|val3/` | `facets_enumeration.whas` | Works but not ideal |

## 🟡 Partially Supported (2 features)

| XSD Feature | Status | Test File | Notes |
|-------------|--------|-----------|-------|
| Attribute groups | Workaround only | `attribute_groups.whas` | Can use type splatting but no dedicated syntax |
| Default values | Attributes only? | `default_fixed_values.whas` | Need to verify model::Attribute support |

## ❌ Not Yet Supported (15 features)

| XSD Feature | Test File | Roadmap Status | Priority |
|-------------|-----------|----------------|----------|
| Namespaces | `namespaces.whas` | Marked as TODO | High |
| xs:any wildcard | `any_wildcard.whas` | Not mentioned | Medium |
| xs:anyAttribute | `any_attribute.whas` | Not mentioned | Medium |
| xs:extension | `extension.whas` | Not mentioned | High |
| xs:union syntax | `union.whas` | Not mentioned | Medium |
| Substitution groups | `substitution_groups.whas` | Not mentioned | Low |
| Identity constraints | `identity_constraints.whas` | Not mentioned | Medium |
| Fixed values | `default_fixed_values.whas` | Related to default values TODO | Medium |
| Nillable elements | `nillable.whas` | Not mentioned | Low |
| Abstract types | `abstract_types.whas` | Not mentioned | Medium |
| Length facets | `facets_length.whas` | Not mentioned | Medium |
| Numeric facets | `facets_numeric.whas` | Not mentioned | Medium |
| whiteSpace facet | `facets_whitespace.whas` | Not mentioned | Low |
| elementFormDefault | `qualified_elements.whas` | Requires namespaces | Medium |
| block/final attributes | `block_final.whas` | Not mentioned | Low |
| xs:notation | `notation.whas` | Not mentioned | Very Low |

## Summary

- **Total XSD features tested**: 38
- **Fully supported**: 21 (55%)
- **Partially supported**: 2 (5%)
- **Not supported**: 15 (40%)

## Notes

### Union Types
The model (`model::SimpleType::Union`) appears to have union type support internally, but there's no WHAS syntax for it. Need to investigate if this is used internally only (e.g., for `IDRefs` which is a list) or if syntax should be added.

### Attribute Groups
Can be simulated using type splatting, but requires creating a dummy element group with attributes. A dedicated attribute group syntax would be cleaner.

### Facets
Only `pattern` (regex) restrictions are supported. All other facets (length, numeric ranges, whitespace) are not yet implemented.

### Inheritance/Extension
No type inheritance mechanism exists. Fields must be manually duplicated between base and derived types.

## Recommendations for XSD Export

When implementing an XSD exporter:

1. **High Priority**: Focus on the 21 fully supported features first
2. **Namespaces**: Critical for real-world XSD - should be prioritized
3. **Extension**: Would significantly improve schema reusability
4. **Additional Facets**: Would improve validation capabilities
5. **Union Syntax**: If model supports it, add syntax

## Test Execution

All 38 tests pass:
- 21 tests verify supported features work correctly
- 2 tests verify partial support
- 15 tests explicitly fail with "not supported" messages

Run tests: `cargo test xsd --lib`

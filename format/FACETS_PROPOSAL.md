# Facet Syntax Proposal

## Problem Statement

WHAS needs syntax for XSD facets (value constraints on simple types) that is clearly distinguished from:
- Occurrence ranges `[]` - structural (how many elements)
- Generic arguments `()` - type-level (type parameters)
- Blocks `{}` - structural (complex type definitions)
- Inheritance `<` - type-level (extends)

## Proposed Solution: Angle Bracket Facets `<>`

Use `<>` with named or shorthand syntax for facets.

### Syntax Rules

```whas
TypeName: BaseType<facets>

// Where facets can be:
// 1. Shorthand range (for common cases)
// 2. Named facets (for specific constraints)
// 3. Mixed (both)
```

## Complete Syntax Examples

### String Length Facets

```whas
// Shorthand range: minLength..maxLength
Username: String<5..20>
Email: String<1..100>
ShortCode: String<3..3>    // exactly 3

// Named syntax
Password: String<minLength: 12>
Description: String<maxLength: 500>
ExactLength: String<length: 8>

// Both
SecurePassword: String<12..128, pattern: /[A-Za-z0-9@#$%]/>
```

### Numeric Range Facets

```whas
// Shorthand: min..max (inclusive by default)
Age: Int<0..150>
Percentage: Float<0.0..100.0>
Port: Int<1..65535>

// Named - inclusive
Score: Int<minInclusive: 0, maxInclusive: 100>
Temperature: Float<minInclusive: -273.15>  // Absolute zero

// Named - exclusive
PositiveInt: Int<minExclusive: 0>           // > 0
Probability: Float<minExclusive: 0.0, maxExclusive: 1.0>  // (0, 1)

// Mixed inclusive/exclusive
Range: Float<minInclusive: 0.0, maxExclusive: 1.0>  // [0, 1)
```

### Numeric Precision Facets

```whas
// Decimal precision
Price: Decimal<totalDigits: 10, fractionDigits: 2>    // 99999999.99
Coordinate: Decimal<totalDigits: 8, fractionDigits: 6>  // 99.999999

// Combined with range
Money: Decimal<0.01..999999.99, fractionDigits: 2>
```

### Whitespace Facet

```whas
// Whitespace handling
Token: String<whiteSpace: "collapse">      // collapse consecutive spaces
PreservedText: String<whiteSpace: "preserve">  // keep all whitespace
NormalizedText: String<whiteSpace: "replace">  // replace with space

// Values: "preserve" | "replace" | "collapse"
```

### Multiple Facets

```whas
// Multiple constraints
SafePassword: String<
    minLength: 12,
    maxLength: 128,
    pattern: /[A-Za-z0-9@#$%^&*]/
>

ValidAge: Int<
    minInclusive: 0,
    maxInclusive: 150
>

Percentage: Float<
    minInclusive: 0.0,
    maxInclusive: 100.0,
    fractionDigits: 2
>
```

## Combined with Other Syntax

```whas
// Facets + Occurrence modifiers
#ages[1..100]: Int<0..150>
// 1-100 elements, each an integer from 0-150

// Facets + Union types
FlexibleId: Int<1..999999> | String<5..50>
// Either integer 1-999999 OR string 5-50 chars

// Facets + Attributes
@score: Int<0..100>
#exam {
    #student: String<1..100>
}

// Facets + Inheritance
BaseId: Int<1..999999>
UserId < BaseId: Int<1000..9999>  // Further restricts range
```

## Shorthand Equivalents

For common patterns, provide shorthand:

```whas
// These are equivalent:
String<5..20>
String<minLength: 5, maxLength: 20>

// These are equivalent:
Int<0..100>
Int<minInclusive: 0, maxInclusive: 100>

// Exact length shorthand:
String<8>
String<length: 8>

// Open-ended ranges:
Int<0..>      // minInclusive: 0, no max
Int<..100>    // no min, maxInclusive: 100
String<5..>   // minLength: 5, no maxLength
```

## Disambiguation Rules

### When `<>` means facets vs inheritance:

```whas
// INHERITANCE: Single identifier after <
ChildType < ParentType { ... }
UserId < BaseId { ... }

// FACETS: Anything else after <
TypeName: BaseType<range>
TypeName: BaseType<facet: value>
TypeName: BaseType<0..100>
```

Context makes it completely unambiguous:
- **Inheritance**: `TypeName < OtherType`
- **Facets**: `TypeName: BaseType<...>`

## Full Example Schema

```whas
// User management schema with facets

// Base types with constraints
Username: String<3..20, pattern: /[a-zA-Z0-9_-]+/>
Email: String<5..100, pattern: /[^@]+@[^@]+\.[^@]+/>
Password: String<12..128>
Age: Int<0..150>
Score: Int<0..100>
Money: Decimal<0.01..999999.99, fractionDigits: 2>

// Phone number with multiple facets
PhoneNumber: String<
    minLength: 10,
    maxLength: 15,
    pattern: /\+?[0-9\s-()]+/
>

// Using faceted types
@id: Int<1..>
@active: Bool
#user {
    #username: Username
    #email: Email
    #age: Age
    #phone?: PhoneNumber

    #scores[0..10]: Score        // 0-10 elements, each score 0-100
    #balance: Money
}

// Inheritance with facets
BaseId: Int<1..999999>
UserId < BaseId: Int<1000..9999>    // Further constrains to 1000-9999

#registered-user {
    @user-id: UserId
    #name: String<1..100>
}
```

## Implementation Notes

### Grammar Changes (schema.pest)

```pest
// Add facet syntax
type_with_facets = { primitive ~ facets }
facets = { "<" ~ facet_list ~ ">" }
facet_list = { facet ~ ("," ~ facet)* }

facet = {
    range_facet |
    named_facet
}

// Shorthand ranges
range_facet = { number ~ ".." ~ number? | ".." ~ number | number }

// Named facets
named_facet = { facet_name ~ ":" ~ facet_value }
facet_name = {
    "minLength" | "maxLength" | "length" |
    "minInclusive" | "maxInclusive" |
    "minExclusive" | "maxExclusive" |
    "totalDigits" | "fractionDigits" |
    "whiteSpace" | "pattern"
}
facet_value = { number | string | regex }
```

### Compiler Changes

```rust
// Parse facets from AST
fn compile_type_with_facets(
    base_type: &ast::Primitive,
    facets: &ast::Facets,
    schema: &mut Schema,
) -> anyhow::Result<TypeRef> {
    let mut restrictions = SimpleTypeRestriction::default();

    for facet in &facets.list {
        match facet {
            Facet::Range(min, max) => {
                // Infer type from base_type
                if base_type.is_string() {
                    restrictions.min_length = Some(min);
                    restrictions.max_length = Some(max);
                } else if base_type.is_numeric() {
                    restrictions.min_inclusive = Some(min);
                    restrictions.max_inclusive = Some(max);
                }
            }
            Facet::Named(name, value) => {
                match name.as_str() {
                    "minLength" => restrictions.min_length = Some(value),
                    "maxLength" => restrictions.max_length = Some(value),
                    // ... etc
                }
            }
        }
    }

    schema.register_simple_type(SimpleType::Derived {
        base: base_type_ref,
        restrictions,
        abstract_type: false,
    })
}
```

### XSD Export

```rust
// In export_simple_type()
fn export_restrictions(&self, restrictions: &SimpleTypeRestriction) -> String {
    let mut xsd = String::new();

    if let Some(len) = restrictions.length {
        xsd.push_str(&format!("      <xs:length value=\"{}\"/>\n", len));
    }
    if let Some(min) = restrictions.min_length {
        xsd.push_str(&format!("      <xs:minLength value=\"{}\"/>\n", min));
    }
    // ... etc for all facets

    xsd
}
```

## Migration Path

### Phase 1: String Length (Easiest)
- Implement `String<min..max>` syntax
- Support `minLength`, `maxLength`, `length` facets
- **Estimated effort**: 2-3 days

### Phase 2: Numeric Ranges
- Implement `Int<min..max>`, `Float<min..max>`
- Support inclusive/exclusive variants
- **Estimated effort**: 2-3 days

### Phase 3: Precision & Whitespace
- Implement `totalDigits`, `fractionDigits`
- Implement `whiteSpace` facet
- **Estimated effort**: 1-2 days

## Benefits

1. **Clear syntax distinction** - Different operators for different concerns
2. **Intuitive** - Angle brackets suggest "constrained type"
3. **Extensible** - Can add more facets without syntax conflicts
4. **Composable** - Works with all other WHAS features
5. **XSD-compatible** - Maps directly to XSD facets
6. **Type-safe** - Facets validated at compile time

## Examples in Use

```whas
// E-commerce schema
ProductCode: String<6, pattern: /[A-Z]{2}[0-9]{4}/>
Price: Decimal<0.01..99999.99, fractionDigits: 2>
Quantity: Int<1..1000>
Rating: Float<1.0..5.0, fractionDigits: 1>

@sku: ProductCode
#product {
    #name: String<1..200>
    #description?: String<maxLength: 2000>
    #price: Price
    #quantity: Quantity
    #rating?: Rating
}

// Configuration schema
Port: Int<1..65535>
Timeout: Int<0..3600>
HostName: String<1..253, pattern: /[a-zA-Z0-9.-]+/>
Percentage: Int<0..100>

#config {
    #host: HostName
    #port: Port
    #timeout: Timeout
    #max-retries: Int<0..10>
    #cache-size: Int<1..>      // at least 1, no max
}
```

## Open Questions

1. Should shorthand ranges be inclusive or exclusive by default?
   - **Proposal**: Inclusive (matches mathematical notation [a, b])

2. Should we allow shorthand for single value?
   - **Proposal**: Yes - `Int<100>` means `maxInclusive: 100`

3. How to handle conflicts between inherited facets?
   - **Proposal**: Derived type must be more restrictive than base

4. Should facets be mandatory in type definitions?
   - **Proposal**: No - facets are optional constraints

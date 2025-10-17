use crate::model;
use anyhow::Result;
use xmltree::{Element, XMLNode};
use std::io::Cursor;

/// Helper trait to add fluent-style methods to xmltree::Element
trait ElementExt {
    fn with_attr(self, key: impl Into<String>, value: impl Into<String>) -> Self;
    fn with_child(self, child: Element) -> Self;
    fn with_prefix(self, prefix: impl Into<String>) -> Self;
}

impl ElementExt for Element {
    fn with_attr(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.attributes.insert(key.into(), value.into());
        self
    }

    fn with_child(mut self, child: Element) -> Self {
        self.children.push(XMLNode::Element(child));
        self
    }

    fn with_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.prefix = Some(prefix.into());
        self
    }
}

use crate::export::Exporter;

/// XSD XML Exporter - exports WHAS model to XSD (XML Schema Definition)
pub struct XsdExporter {
    /// Target namespace (if supported)
    target_namespace: Option<String>,
}

impl Default for XsdExporter {
    fn default() -> Self {
        Self {
            target_namespace: None,
        }
    }
}

impl Exporter for XsdExporter {
    type Output = String;

    fn export_schema(self, schema: &model::Schema) -> Result<Self::Output> {
        // Build xs:schema root element
        let mut schema_elem = Element::new("schema")
            .with_prefix("xs")
            .with_attr("xmlns:xs", "http://www.w3.org/2001/XMLSchema")
            .with_attr("elementFormDefault", "qualified");

        if let Some(ns) = &self.target_namespace {
            schema_elem = schema_elem.with_attr("targetNamespace", ns);
        }

        // Export simple types (primitives are built into XSD, only custom types need export)
        // Sort type names for deterministic output
        let mut type_names = schema.all_type_names();
        type_names.sort();

        for type_name in &type_names {
            if let Some(simple_type) = schema.get_simpletype_by_name(type_name) {
                if !simple_type.is_builtin() {
                    schema_elem = schema_elem.with_child(self.export_simple_type(type_name, simple_type, schema)?);
                }
            }
        }

        // Export complex types (groups) - sorted for deterministic output
        for type_name in &type_names {
            if let Some(group) = schema.get_group_by_name(type_name) {
                schema_elem = schema_elem.with_child(self.export_complex_type(type_name, group, schema)?);
            }
        }

        // Export top-level elements (sorted by name for deterministic output)
        // Elements are named by their name() method, not via the type name mapping
        let mut root_elements = schema.get_elements_root();
        root_elements.sort_by_key(|el| el.name());

        for element in &root_elements {
            schema_elem = schema_elem.with_child(self.export_element(element.name(), element, schema)?);
        }

        // Write XML to string with declaration
        let mut buffer = Cursor::new(Vec::new());
        schema_elem.write(&mut buffer)?;

        let xml_bytes = buffer.into_inner();
        let xml_content = String::from_utf8(xml_bytes)?;

        // Prepend XML declaration
        Ok(format!("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n{}", xml_content))
    }
}

impl XsdExporter {
    pub fn with_namespace(namespace: impl Into<String>) -> Self {
        Self {
            target_namespace: Some(namespace.into()),
        }
    }

    fn export_simple_type(
        &self,
        name: &str,
        simple_type: &model::SimpleType,
        schema: &model::Schema,
    ) -> Result<Element> {
        let mut simple_type_elem = Element::new("xs:simpleType")
            .with_attr("name", name);

        match simple_type {
            model::SimpleType::Derived { base, restrictions, .. } => {
                let base_name = base.resolve(schema).to_type_name(schema);
                let mut restriction_elem = Element::new("xs:restriction")
                    .with_attr("base", format!("xs:{}", self.map_primitive_to_xsd(&base_name)));

                // Export all facets using helper
                for facet_elem in self.export_restrictions(restrictions)? {
                    restriction_elem = restriction_elem.with_child(facet_elem);
                }

                simple_type_elem = simple_type_elem.with_child(restriction_elem);
            }
            model::SimpleType::Union { member_types } => {
                let members: Vec<String> = member_types
                    .iter()
                    .map(|t| {
                        let type_name = t.resolve(schema).to_type_name(schema);
                        format!("xs:{}", self.map_primitive_to_xsd(&type_name))
                    })
                    .collect();

                simple_type_elem = simple_type_elem.with_child(
                    Element::new("xs:union")
                        .with_attr("memberTypes", members.join(" "))
                        
                );
            }
            model::SimpleType::List { item_type, separator: _ } => {
                let item_name = item_type.resolve(schema).to_type_name(schema);
                simple_type_elem = simple_type_elem.with_child(
                    Element::new("xs:list")
                        .with_attr("itemType", format!("xs:{}", self.map_primitive_to_xsd(&item_name)))
                        
                );
            }
            model::SimpleType::Builtin { .. } => {
                // Should not reach here - builtins are filtered out
            }
        }

        Ok(simple_type_elem)
    }

    fn export_complex_type(
        &self,
        name: &str,
        group: &model::Group,
        schema: &model::Schema,
    ) -> Result<Element> {
        let mut complex_type_elem = Element::new("xs:complexType")
            .with_attr("name", name);

        // Add abstract attribute if type is abstract
        if group.is_abstract() {
            complex_type_elem = complex_type_elem.with_attr("abstract", "true");
        }

        // Handle inheritance with xs:extension
        if let Some(base_ref) = group.base_type() {
            // Find the base type name
            if let Some(base_name) = schema.get_type_name_for_group(base_ref) {
                let mut extension_elem = Element::new("xs:extension")
                    .with_attr("base", base_name);

                // Export only local fields (not inherited)
                extension_elem = extension_elem.with_child(self.export_group_content_local(group, schema)?);

                let complex_content_elem = Element::new("xs:complexContent")
                    .with_child(extension_elem);

                complex_type_elem = complex_type_elem.with_child(complex_content_elem);
            } else {
                // Fallback if base name not found - export all content
                complex_type_elem = complex_type_elem.with_child(self.export_group_content(group, schema)?);
            }
        } else {
            // No inheritance - export group content normally
            complex_type_elem = complex_type_elem.with_child(self.export_group_content(group, schema)?);
        }

        Ok(complex_type_elem)
    }

    fn export_group_content(
        &self,
        group: &model::Group,
        schema: &model::Schema,
    ) -> Result<Element> {
        // Determine group type
        let group_tag = match group.ty() {
            model::GroupType::Sequence => "xs:sequence",
            model::GroupType::Choice => "xs:choice",
            model::GroupType::All => "xs:all",
        };

        let mut group_elem = Element::new(group_tag);

        // Export items
        for item in group.items() {
            match item {
                model::GroupItem::Element(el_ref) => {
                    let element = el_ref.resolve(schema);
                    group_elem = group_elem.with_child(self.export_element_inline(element, schema)?);
                }
                model::GroupItem::Group(g_ref) => {
                    let nested_group = g_ref.resolve(schema);
                    group_elem = group_elem.with_child(self.export_group_content(nested_group, schema)?);
                }
            }
        }

        Ok(group_elem)
    }

    /// Export only local group content (excludes inherited fields from base type)
    fn export_group_content_local(
        &self,
        group: &model::Group,
        schema: &model::Schema,
    ) -> Result<Element> {
        // Determine group type
        let group_tag = match group.ty() {
            model::GroupType::Sequence => "xs:sequence",
            model::GroupType::Choice => "xs:choice",
            model::GroupType::All => "xs:all",
        };

        let mut group_elem = Element::new(group_tag);

        // Export only local items (all items in this group are local by definition)
        // Inheritance is handled by XSD's extension mechanism
        for item in group.items() {
            match item {
                model::GroupItem::Element(el_ref) => {
                    let element = el_ref.resolve(schema);
                    group_elem = group_elem.with_child(self.export_element_inline(element, schema)?);
                }
                model::GroupItem::Group(g_ref) => {
                    let nested_group = g_ref.resolve(schema);
                    group_elem = group_elem.with_child(self.export_group_content(nested_group, schema)?);
                }
            }
        }

        Ok(group_elem)
    }

    fn export_element(
        &self,
        name: &str,
        element: &model::Element,
        schema: &model::Schema,
    ) -> Result<Element> {
        let mut elem = Element::new("xs:element")
            .with_attr("name", name);

        // Add occurrence constraints
        elem = elem.with_attr("minOccurs", element.min_occurs().to_string());
        if let Some(max) = element.max_occurs() {
            elem = elem.with_attr("maxOccurs", max.to_string());
        } else {
            elem = elem.with_attr("maxOccurs", "unbounded");
        }

        // Get attributes
        let attrs = element.group_merged_attributes(schema);
        let has_attrs = !attrs.as_vec().is_empty();

        // Check if it has complex type
        if let Some(group_type) = element.typing().grouptype(schema) {
            // Check for mixed content
            let mut complex_type_elem = Element::new("xs:complexType");
            if element.is_mixed_content(schema) {
                complex_type_elem = complex_type_elem.with_attr("mixed", "true");
            }

            complex_type_elem = complex_type_elem.with_child(self.export_group_content(group_type, schema)?);

            // Add attributes
            for attr_elem in self.export_attributes(&attrs, schema)? {
                complex_type_elem = complex_type_elem.with_child(attr_elem);
            }

            elem = elem.with_child(complex_type_elem);
        } else if let model::TypeRef::Simple(simple_ref) = element.typing() {
            let simple_type = simple_ref.resolve(schema);

            // Check if this is an anonymous union (inline union)
            let is_anonymous_union = matches!(simple_type, model::SimpleType::Union { .. }) &&
                                     schema.get_type_name_for_simpletype(simple_ref).is_none();

            // Simple type with attributes (simpleContent)
            if has_attrs {
                let mut complex_type_elem = Element::new("xs:complexType");
                let mut simple_content_elem = Element::new("xs:simpleContent");

                if is_anonymous_union {
                    // For anonymous unions with attributes, we need to define the union inline
                    // Use xs:restriction with an inline simpleType
                    let mut restriction_elem = Element::new("xs:restriction");
                    restriction_elem = restriction_elem.with_child(self.export_simple_type_inline(simple_type, schema)?);

                    // Add attributes
                    for attr_elem in self.export_attributes(&attrs, schema)? {
                        restriction_elem = restriction_elem.with_child(attr_elem);
                    }

                    simple_content_elem = simple_content_elem.with_child(restriction_elem);
                } else {
                    // Named type - use extension
                    let type_name = self.get_simple_type_xsd_name(simple_ref, schema);
                    let mut extension_elem = Element::new("xs:extension")
                        .with_attr("base", type_name);

                    // Add attributes
                    for attr_elem in self.export_attributes(&attrs, schema)? {
                        extension_elem = extension_elem.with_child(attr_elem);
                    }

                    simple_content_elem = simple_content_elem.with_child(extension_elem);
                }

                complex_type_elem = complex_type_elem.with_child(simple_content_elem);
                elem = elem.with_child(complex_type_elem);
            } else {
                // Simple type without attributes
                if is_anonymous_union {
                    // Export inline union
                    elem = elem.with_child(self.export_simple_type_inline(simple_type, schema)?);
                } else {
                    let type_name = self.get_simple_type_xsd_name(simple_ref, schema);
                    elem = elem.with_attr("type", type_name);
                }
            }
        } else if has_attrs {
            // Attributes only (empty content)
            let mut complex_type_elem = Element::new("xs:complexType");

            // Add attributes
            for attr_elem in self.export_attributes(&attrs, schema)? {
                complex_type_elem = complex_type_elem.with_child(attr_elem);
            }

            elem = elem.with_child(complex_type_elem);
        } else {
        }

        Ok(elem)
    }

    fn export_attributes(
        &self,
        attrs: &model::Attributes,
        schema: &model::Schema,
    ) -> Result<Vec<Element>> {
        // Sort attributes by name for deterministic output
        let mut attr_vec = attrs.as_vec().clone();
        attr_vec.sort_by_key(|attr_ref| attr_ref.resolve(schema).name());

        let mut result = Vec::new();

        for attr_ref in attr_vec {
            let attr = attr_ref.resolve(schema);
            let mut attr_elem = Element::new("xs:attribute")
                .with_attr("name", attr.name());

            // Type - attr.typing is directly a Ref<SimpleType>
            let attr_type = attr.typing.resolve(schema);

            // Check if this is an anonymous union type (inline union)
            if matches!(attr_type, model::SimpleType::Union { .. }) &&
               schema.get_type_name_for_simpletype(&attr.typing).is_none() {
                // Anonymous inline union - export inline
                attr_elem = attr_elem.with_child(self.export_simple_type_inline(attr_type, schema)?);
            } else {
                // Named type or non-union - use type reference
                let type_name = self.get_simple_type_xsd_name(&attr.typing, schema);
                attr_elem = attr_elem.with_attr("type", type_name);

                // Required/optional (use="required" vs use="optional")
                if *attr.required() {
                    attr_elem = attr_elem.with_attr("use", "required");
                } // Optional is the default, no need to specify

            }

            result.push(attr_elem);
        }

        Ok(result)
    }

    fn export_element_inline(
        &self,
        element: &model::Element,
        schema: &model::Schema,
    ) -> Result<Element> {
        let mut elem = Element::new("xs:element")
            .with_attr("name", element.name());

        // Occurrence constraints
        elem = elem.with_attr("minOccurs", element.min_occurs().to_string());
        if let Some(max) = element.max_occurs() {
            elem = elem.with_attr("maxOccurs", max.to_string());
        } else {
            elem = elem.with_attr("maxOccurs", "unbounded");
        }

        // Type reference
        if let model::TypeRef::Simple(simple_ref) = element.typing() {
            let simple_type = simple_ref.resolve(schema);

            // Check if this is an anonymous type (inline facets or inline unions)
            if schema.get_type_name_for_simpletype(simple_ref).is_none() &&
               (simple_type.is_derived() || matches!(simple_type, model::SimpleType::Union { .. })) {
                // Export as anonymous inline simpleType
                elem = elem.with_child(self.export_simple_type_inline(simple_type, schema)?);
            } else {
                // Named type reference
                let type_name = self.get_simple_type_xsd_name(simple_ref, schema);
                elem = elem.with_attr("type", type_name);
            }
        } else if let Some(group_type) = element.typing().grouptype(schema) {
            let mut complex_type_elem = Element::new("xs:complexType");
            complex_type_elem = complex_type_elem.with_child(self.export_group_content(group_type, schema)?);
            elem = elem.with_child(complex_type_elem);
        } else {
        }

        Ok(elem)
    }

    /// Export an inline anonymous simpleType (for inline facets or inline unions)
    fn export_simple_type_inline(
        &self,
        simple_type: &model::SimpleType,
        schema: &model::Schema,
    ) -> Result<Element> {
        let mut simple_type_elem = Element::new("xs:simpleType");

        match simple_type {
            model::SimpleType::Derived { base, restrictions, .. } => {
                let base_name = base.resolve(schema).to_type_name(schema);
                let mut restriction_elem = Element::new("xs:restriction")
                    .with_attr("base", format!("xs:{}", self.map_primitive_to_xsd(&base_name)));

                for facet_elem in self.export_restrictions(restrictions)? {
                    restriction_elem = restriction_elem.with_child(facet_elem);
                }

                simple_type_elem = simple_type_elem.with_child(restriction_elem);
            }
            model::SimpleType::Union { member_types } => {
                let members: Vec<String> = member_types
                    .iter()
                    .map(|t| {
                        let type_name = self.get_simple_type_xsd_name(t, schema);
                        // Only add xs: prefix if not already present
                        if type_name.starts_with("xs:") {
                            type_name
                        } else {
                            format!("xs:{}", self.map_primitive_to_xsd(&type_name))
                        }
                    })
                    .collect();

                simple_type_elem = simple_type_elem.with_child(
                    Element::new("xs:union")
                        .with_attr("memberTypes", members.join(" "))
                        
                );
            }
            _ => {
                // Shouldn't happen for inline types, but handle gracefully
            }
        }

        Ok(simple_type_elem)
    }

    /// Export restriction facets (helper for reuse)
    fn export_restrictions(
        &self,
        restrictions: &model::restriction::SimpleTypeRestriction,
    ) -> Result<Vec<Element>> {
        let mut facets = Vec::new();

        // Enumeration facet
        if let Some(enumeration) = restrictions.enumeration.as_ref() {
            for value in enumeration {
                facets.push(
                    Element::new("xs:enumeration")
                        .with_attr("value", value)
                        
                );
            }
        }

        // Length facets
        if let Some(length) = restrictions.length {
            facets.push(
                Element::new("xs:length")
                    .with_attr("value", length.to_string())
                    
            );
        }
        if let Some(min_length) = restrictions.min_length {
            facets.push(
                Element::new("xs:minLength")
                    .with_attr("value", min_length.to_string())
                    
            );
        }
        if let Some(max_length) = restrictions.max_length {
            facets.push(
                Element::new("xs:maxLength")
                    .with_attr("value", max_length.to_string())
                    
            );
        }

        // Pattern facet
        if let Some(pattern) = restrictions.pattern.as_ref() {
            facets.push(
                Element::new("xs:pattern")
                    .with_attr("value", pattern)
                    
            );
        }

        // Whitespace facet
        if let Some(white_space) = restrictions.white_space {
            let ws_value = match white_space {
                model::restriction::WhiteSpaceHandling::Preserve => "preserve",
                model::restriction::WhiteSpaceHandling::Replace => "replace",
                model::restriction::WhiteSpaceHandling::Collapse => "collapse",
            };
            facets.push(
                Element::new("xs:whiteSpace")
                    .with_attr("value", ws_value)
                    
            );
        }

        // Numeric range facets
        if let Some(min_inclusive) = restrictions.min_inclusive.as_ref() {
            facets.push(
                Element::new("xs:minInclusive")
                    .with_attr("value", min_inclusive)
                    
            );
        }
        if let Some(max_inclusive) = restrictions.max_inclusive.as_ref() {
            facets.push(
                Element::new("xs:maxInclusive")
                    .with_attr("value", max_inclusive)
                    
            );
        }
        if let Some(min_exclusive) = restrictions.min_exclusive.as_ref() {
            facets.push(
                Element::new("xs:minExclusive")
                    .with_attr("value", min_exclusive)
                    
            );
        }
        if let Some(max_exclusive) = restrictions.max_exclusive.as_ref() {
            facets.push(
                Element::new("xs:maxExclusive")
                    .with_attr("value", max_exclusive)
                    
            );
        }

        // Decimal precision facets
        if let Some(total_digits) = restrictions.total_digits {
            facets.push(
                Element::new("xs:totalDigits")
                    .with_attr("value", total_digits.to_string())
                    
            );
        }
        if let Some(fraction_digits) = restrictions.fraction_digits {
            facets.push(
                Element::new("xs:fractionDigits")
                    .with_attr("value", fraction_digits.to_string())
                    
            );
        }

        Ok(facets)
    }

    /// Get the XSD type name for a simple type reference
    /// Checks if the type has a custom name in the schema, otherwise returns the primitive type name
    fn get_simple_type_xsd_name(&self, simple_ref: &model::Ref<model::SimpleType>, schema: &model::Schema) -> String {
        let simple_type = simple_ref.resolve(schema);

        // Check if this is a builtin - builtins always use xs: prefix even if registered in schema
        if simple_type.is_builtin() {
            let base_name = simple_type.to_type_name(schema);
            return format!("xs:{}", self.map_primitive_to_xsd(&base_name));
        }

        // Check if this type has a custom name (like "FlexibleId")
        if let Some(custom_name) = schema.get_type_name_for_simpletype(simple_ref) {
            return custom_name;
        }

        // Otherwise, get the primitive base type and map to XSD
        let base_name = simple_type.to_type_name(schema);
        format!("xs:{}", self.map_primitive_to_xsd(&base_name))
    }

    /// Map WHAS primitive type names to XSD type names
    fn map_primitive_to_xsd(&self, whas_type: &str) -> String {
        match whas_type {
            "String" => "string",
            "Int" | "Integer" => "integer",
            "Bool" | "Boolean" => "boolean",
            "Date" => "date",
            "DateTime" => "dateTime",
            "DateTimestamp" => "dateTime",
            "Time" => "time",
            "Duration" => "duration",
            "Float" => "float",
            "Double" => "double",
            "Short" => "short",
            "Decimal" => "decimal",
            "ID" => "ID",
            "IDRef" => "IDREF",
            "IDRefs" => "IDREFS",
            "URI" => "anyURI",
            "Lang" => "language",
            "Name" => "Name",
            "NoColName" => "NCName",
            "-Int" => "negativeInteger",
            "+Int" => "nonNegativeInteger",
            "Token" => "token",
            "NameToken" => "NMTOKEN",
            "NameTokens" => "NMTOKENS",
            _ => whas_type, // Custom type, use as-is
        }.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_primitive_mapping() {
        let exporter = XsdExporter::default();
        assert_eq!(exporter.map_primitive_to_xsd("String"), "string");
        assert_eq!(exporter.map_primitive_to_xsd("Int"), "integer");
        assert_eq!(exporter.map_primitive_to_xsd("Bool"), "boolean");
        assert_eq!(exporter.map_primitive_to_xsd("Date"), "date");
        assert_eq!(exporter.map_primitive_to_xsd("URI"), "anyURI");
    }
}

use super::*;

/// Facet constraints on a type using <> syntax
/// Example: String<5..20, pattern: /regex/>
#[derive(Debug, Eq, PartialEq, Clone, FromPest)]
#[pest_ast(rule(Rule::facets))]
pub struct Facets {
    pub items: Option<FacetList>,
}

#[derive(Debug, Eq, PartialEq, Clone, FromPest)]
#[pest_ast(rule(Rule::facet_list))]
pub struct FacetList {
    pub items: Vec<FacetItem>,
}

#[derive(Debug, Eq, PartialEq, Clone, FromPest)]
#[pest_ast(rule(Rule::facet_item))]
pub enum FacetItem {
    Shorthand(FacetShorthand),
    Named(FacetNamed),
}

/// Shorthand facet syntax
/// Examples:
/// - 5..20 (min..max)
/// - 5 (exact)
/// - 5.. (min only)
/// - ..20 (max only)
#[derive(Debug, Eq, PartialEq, Clone, FromPest)]
#[pest_ast(rule(Rule::facet_shorthand))]
pub struct FacetShorthand {
    #[pest_ast(outer(with(span_into_str), with(str::to_string)))]
    pub value: String,
}

impl FacetShorthand {
    /// Parse the shorthand into min/max values
    /// Returns (min, max) where None means unbounded
    pub fn parse_range(&self) -> FacetRange {
        let text = self.value.trim();

        if text.contains("..") {
            let parts: Vec<&str> = text.split("..").collect();
            match (parts.get(0), parts.get(1)) {
                (Some(&""), Some(max)) if !max.is_empty() => {
                    // ..max
                    FacetRange {
                        min: None,
                        max: Some(max.to_string()),
                    }
                }
                (Some(min), Some(&"")) if !min.is_empty() => {
                    // min..
                    FacetRange {
                        min: Some(min.to_string()),
                        max: None,
                    }
                }
                (Some(min), Some(max)) if !min.is_empty() && !max.is_empty() => {
                    // min..max
                    FacetRange {
                        min: Some(min.to_string()),
                        max: Some(max.to_string()),
                    }
                }
                _ => FacetRange {
                    min: None,
                    max: None,
                },
            }
        } else {
            // Exact value
            FacetRange {
                min: Some(text.to_string()),
                max: Some(text.to_string()),
            }
        }
    }
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct FacetRange {
    pub min: Option<String>,
    pub max: Option<String>,
}

/// Named facet syntax
/// Example: minLength: 5
#[derive(Debug, Eq, PartialEq, Clone, FromPest)]
#[pest_ast(rule(Rule::facet_named))]
pub struct FacetNamed {
    pub name: FacetName,
    pub value: FacetValue,
}

#[derive(Debug, Eq, PartialEq, Clone, FromPest)]
#[pest_ast(rule(Rule::facet_name))]
pub struct FacetName {
    #[pest_ast(outer(with(span_into_str), with(str::to_string)))]
    pub value: String,
}

impl FacetName {
    pub fn as_str(&self) -> &str {
        &self.value
    }
}

#[derive(Debug, Eq, PartialEq, Clone, FromPest)]
#[pest_ast(rule(Rule::facet_value))]
pub enum FacetValue {
    Regex(TypeRegex),
    String(AttrItemStr),
    Number(Number),
}

impl FacetValue {
    pub fn as_string(&self) -> String {
        match self {
            FacetValue::Regex(r) => r.value.clone(),
            FacetValue::String(s) => {
                // Remove quotes
                let text = &s.value;
                text.trim_matches(|c| c == '"' || c == '\'' || c == '`' || c == '%')
                    .to_string()
            }
            FacetValue::Number(n) => n.value.clone(),
        }
    }
}

#[derive(Debug, Eq, PartialEq, Clone, FromPest)]
#[pest_ast(rule(Rule::number))]
pub struct Number {
    #[pest_ast(outer(with(span_into_str), with(str::to_string)))]
    pub value: String,
}

impl Number {
    pub fn as_str(&self) -> &str {
        &self.value
    }

    pub fn parse_int(&self) -> Option<i64> {
        self.as_str().parse().ok()
    }

    pub fn parse_uint(&self) -> Option<usize> {
        self.as_str().parse().ok()
    }

    pub fn parse_float(&self) -> Option<f64> {
        self.as_str().parse().ok()
    }
}

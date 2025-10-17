use regex::Regex;
use serde::{Deserialize, Serialize};

/// XSD whiteSpace facet handling modes
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WhiteSpaceHandling {
    /// Preserve all whitespace (tabs, newlines, spaces)
    Preserve,
    /// Replace each tab, newline, and carriage return with a single space
    Replace,
    /// Replace sequences of whitespace with single space, trim leading/trailing
    Collapse,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize, Default)]
pub struct SimpleTypeRestriction {
    /// Specifies the exact number of characters or list items allowed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub length: Option<usize>,
    /// Specifies the minimum number of characters or list items allowed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_length: Option<usize>,
    /// Specifies the maximum number of characters or list items allowed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_length: Option<usize>,
    /// Defines a regular expression pattern that the value must match.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pattern: Option<String>,
    // cant do actual Regex type here because it is not Eq or Serialize
    /// Specifies a list of acceptable values for the simple type.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enumeration: Option<Vec<String>>,
    /// Specifies how whitespace should be handled (preserve, replace, collapse).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub white_space: Option<WhiteSpaceHandling>,
    /// Specifies the minimum value for the element (stored as string to support decimals).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_inclusive: Option<String>,
    /// Specifies the maximum value for the element (stored as string to support decimals).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_inclusive: Option<String>,
    /// Specifies the minimum value for the element (stored as string to support decimals).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_exclusive: Option<String>,
    /// Specifies the maximum value for the element (stored as string to support decimals).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_exclusive: Option<String>,
    /// Specifies the total number of digits that can appear in the element.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_digits: Option<usize>,
    /// Specifies the maximum number of decimal places that can appear in the element.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fraction_digits: Option<usize>,
}

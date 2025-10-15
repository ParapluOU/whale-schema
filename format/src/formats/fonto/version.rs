use serde::{Deserialize, Serialize};
use tap::Tap;

#[derive(Clone, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub struct FontoVersion(Vec<usize>);

impl FontoVersion {
    pub fn try_from_str(s: &str) -> anyhow::Result<Self> {
        Ok(FontoVersion(
            s.split('.')
                .map(|s| s.parse::<usize>())
                .collect::<Result<Vec<usize>, _>>()?,
        ))
    }

    /// todo: expand to all versions
    pub fn min_schema_compiler_version(&self) -> FontoSchemaCompilerVersion {
        if self.is_8_8() {
            FontoSchemaCompilerVersion(vec![2, 3, 2])
        } else {
            FontoSchemaCompilerVersion(vec![2, 3, 3])
        }
    }

    pub fn is_8(&self) -> bool {
        self.0[0] == 8
    }

    pub fn is_8_8(&self) -> bool {
        self.0[0] == 8 && self.0[1] == 8
    }

    pub fn is_7(&self) -> bool {
        self.0[0] == 7
    }
}

impl Default for FontoVersion {
    fn default() -> Self {
        Self(vec![8, 8, 0])
    }
}

#[derive(Clone, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub struct FontoSchemaCompilerVersion(Vec<usize>);

impl Default for FontoSchemaCompilerVersion {
    fn default() -> Self {
        Self(vec![2, 3, 2]) // 8.8.0
    }
}

impl FontoSchemaCompilerVersion {
    pub fn try_from_str(s: &str) -> anyhow::Result<Self> {
        Ok(Self(
            s.split('.')
                .map(|s| s.parse::<usize>())
                .collect::<Result<Vec<usize>, _>>()?,
        ))
    }
}

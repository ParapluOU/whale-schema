use crate::model;
use std::collections::HashSet;
use std::ops::Deref;

/// cotainer that embodies the result of a compilation sub-process.
/// for example, compiling and resolving all types for a single element, or block or attribute
/// todo: phase out in favor of mutable Schema
pub struct CompileResult<T> {
    /// the target thing that should come out of a compilation process
    pub target: T,
    /// all user types that are needed for the definition of the thing above
    pub supporting_types_defined: HashSet<model::Type>,
    /// all implicit system-generated (anonymous) types that are needed for the definition of the thing above
    pub supporting_types_implicit: HashSet<model::Type>,
}

impl<T> CompileResult<T> {
    pub fn map<X>(
        self,
        mut mapper: impl FnMut(T) -> anyhow::Result<X>,
    ) -> anyhow::Result<CompileResult<X>> {
        Ok(CompileResult {
            target: mapper(self.target)?,
            supporting_types_defined: self.supporting_types_defined,
            supporting_types_implicit: self.supporting_types_implicit,
        })
    }

    pub fn unwrap(self) -> T {
        self.target
    }
}

impl<T> Deref for CompileResult<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.target
    }
}

impl<T> From<T> for CompileResult<T> {
    fn from(target: T) -> Self {
        Self {
            target,
            supporting_types_defined: Default::default(),
            supporting_types_implicit: Default::default(),
        }
    }
}

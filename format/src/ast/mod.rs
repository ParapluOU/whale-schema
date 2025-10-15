// pub use crate::*;
pub use {
    crate::{Rule, WHASParser},
    anyhow::{anyhow, Context},
    enum_variant_macros::FromVariants,
    from_pest::FromPest,
    pest::{Parser, Span},
    pest_ast::FromPest,
    std::{
        cmp::Ordering,
        convert::identity,
        fmt::Formatter,
        ops::{Deref, Range},
        path::{Path, PathBuf},
    },
};

pub(crate) use {crate::ast, crate::default, crate::model};

mod argvars;
mod attrs;
mod blocks;
mod comments;
mod elements;
mod file;
mod idents;
mod imports;
mod keywords;
mod primitives;
mod regex;
mod schemas;
mod splats;
mod symbols;
mod typedefs;
mod types;
mod typings;

pub use {
    argvars::*, attrs::*, blocks::*, comments::*, elements::*, file::*, idents::*, imports::*,
    keywords::*, primitives::*, regex::*, schemas::*, splats::*, symbols::*, typedefs::*, types::*,
    typings::*,
};

// todo: adjust this so we can store the spans in the AST nodes,
// so we can later provide better feedback on parsing errors and their locations
fn span_into_str(span: Span) -> &str {
    // panic!("{:#?}", &span);
    span.as_str()
}

fn strip_delimiters(s: &str) -> String {
    // Assuming value is always enclosed in '/' tokens
    s[1..s.len() - 1].to_string()
}

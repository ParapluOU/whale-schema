#![feature(associated_type_bounds)]
#![feature(let_chains)]
#![feature(try_blocks)]
#![feature(absolute_path)]

mod ast;
mod compiler;
mod export;
mod formats;
mod import;
pub mod model;
mod sourced;
pub(crate) mod tests;
mod tools;
mod validation;

use pest_derive::Parser;
pub(crate) use tools::default;
pub use {crate::model::*, validation::*};

#[derive(Parser)]
#[grammar = "../schema.pest"] // relative to src
pub struct WHASParser;

#[test]
fn it_compiles() {}

#![feature(box_patterns, box_syntax, test)]
mod cli;
mod data;
mod grammar;
mod parse;
mod reduce;

pub use cli::run;
pub use data::{bool, church};
pub use grammar::{Defn, File, Term};
// TODO: we should expose our own error type
pub use parse::{to_file, to_term, ParserResult};

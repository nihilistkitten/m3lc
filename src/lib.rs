mod cli;
mod grammar;
mod parse;
mod reduce;

pub use cli::run;
pub use grammar::{Defn, File, Term};
// TODO: we should expose our own error type
pub use parse::{to_file, to_term, ParserResult};

//! The command-line interface.

use std::{fs, path::PathBuf};

use crate::{to_file, ParserResult, Term};
use colored::Colorize;
use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(rename_all = "kebab-case")]
struct Opt {
    /// Input file
    #[structopt(parse(from_os_str))]
    file: PathBuf,
}

impl Term {
    /// Guess the value of the term.
    ///
    /// Currently, only supports Church numerals.
    fn guess_val(self) -> Option<String> {
        self.try_into()
            .ok()
            .map(|n: usize| format!("Church numeral {}", n))
    }
}

/// Run the CLI.
///
/// # Errors
/// Returns `ParserResult` if passed an invalid term.
pub fn run() -> ParserResult<()> {
    let opt = Opt::from_args();

    let contents = fs::read_to_string(opt.file).expect("Something went wrong reading the file");
    let input = to_file(&contents)?;

    let output = input.unroll().reduce();
    println!("{}", &output);
    if let Some(s) = output.guess_val() {
        println!("I think this is: {}", s.bold());
    }
    Ok(())
}

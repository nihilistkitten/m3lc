//! The command-line interface.

use std::{fmt::Display, fs};

use crate::{to_file, ParserResult, Term};
use colored::{ColoredString, Colorize};
use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(rename_all = "kebab-case")]
struct Opt {
    /// Input file
    file: String,

    /// Print each beta-reduction step
    #[structopt(short, long)]
    verbose: bool,
}

impl Term {
    /// Guess the value of the term.
    ///
    /// Currently, supports Church numerals and booleans.
    fn guess_val(&self) -> Matches {
        vec![
            self.try_into()
                .ok()
                .map(|n: usize| format!("Church numeral {}", n)),
            self.try_into().ok().map(|b: bool| format!("boolean {}", b)),
        ]
        .into_iter()
        .flatten()
        .map(|s| s.green())
        .collect()
    }
}

struct Matches {
    matches: Vec<ColoredString>,
}

impl Matches {
    pub fn is_empty(&self) -> bool {
        self.matches.is_empty()
    }
}

impl FromIterator<ColoredString> for Matches {
    fn from_iter<T: IntoIterator<Item = ColoredString>>(iter: T) -> Self {
        Self {
            matches: iter.into_iter().collect(),
        }
    }
}

impl Display for Matches {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut out = String::new();
        if self.matches.len() == 1 {
            out += &self.matches[0].to_string();
        } else {
            for val in &self.matches {
                out += "\n";
                out += " - ";
                out += &val.to_string();
            }
        }
        write!(f, "{}", out)
    }
}

/// Run the CLI.
///
/// # Errors
/// Returns `ParserResult` if passed an invalid term.
pub fn run() -> ParserResult<()> {
    let opt = Opt::from_args();

    let contents = fs::read_to_string(&opt.file).expect("Unable to open file");
    let input = to_file(&contents)?;

    let output = input.unroll().reduce(opt.verbose);
    println!("{}", &output);

    let guessed_value = output.guess_val();
    if !guessed_value.is_empty() {
        println!();
        println!("Alpha-equivalent to: {}", guessed_value);
    }
    Ok(())
}

// TODO: test this lol

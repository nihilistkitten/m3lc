//! The command-line interface.

use std::{fs, path::PathBuf};

use crate::parse::{to_file, ParserResult};
use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(rename_all = "kebab-case")]
struct Opt {
    /// Input file
    #[structopt(parse(from_os_str))]
    file: PathBuf,
}

pub fn run() -> ParserResult<()> {
    let opt = Opt::from_args();

    let contents = fs::read_to_string(opt.file).expect("Something went wrong reading the file");
    let input = to_file(&contents)?;

    println!("input: {}", &input);

    let output = input.unroll().reduce();
    println!("output: {}", &output);
    Ok(())
}

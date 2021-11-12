A parser for "ML-Like Lambda Calculus" (mlllc, or m3lc, hence my not-so-clever
name for source files).

## Architecture

Each piece of the assignment corresponds (roughly) to one of the files.

1. The parser: this is handled by the `parse.rs` file and the `m3lc.pest`
   grammar. It's built on top of the `pest` and `pest_consume` crates, which
   together allow the strongly typed, recursive pattern-matching on terms that
   you see consumed by the `pest_consume::parser` proc macro.
2. The term: the Rust types corresponding to the AST for a `.m3lc` file is
   defined in the `grammar.rs` file. This file is fairly self-explanatory. It
   does two-pieces of non-trivial work. First, it "unrolls" a `File` into a
   `Term` via applying defined terms, in reverse order, as lambda abstractions
   over the `main` term. Second, it implements `Display` for the AST types,
   which allows printing them as readable `.m3lc` code.
3. Reduction: the `reduce.rs` file implements the public methods `reduce` and
   `alpha_equiv` on `Term`, which perform (normal order) beta-reduction and
   check alpha-equivalence, respectively. (Alpha-equivalence was not strictly
   part of the assignment, but is useful for automated testing of normal order
   beta-reduction, which only guarantees normalization up to alpha-renaming.)
4. Testing: the simple tests are implemented as unit tests in the various
   files. A common idiom here is to use a Rust macro to allow writing test code
   generic over the term to be tested. The longer-form tests are in the
   `examples/` directory and are not tested automatically.
5. Specific terms: each term is implmented in a separate file in the
   `examples/` directory in the project root.
6. Debug mode: this is implemented as a `-v` (for verbose) flag for the CLI,
   parsed by the `structopt` crate in `cli.rs`. For full documentation of the
   CLI, pass the `-h` flag.

## Extras

There are a couple of "extra" things I implemented beyond the problem
assignment; I figured I'd document them here:

1. Alpha-equivalence: as mentioned, this is useful for unit testing, but is
   also critical for #2.
2. Term inference: if given the `-i` command-line flag, the CLI checks for
   alpha-equivalence of the final output to boolean and church numeral types.
   This work is handled by the `guess_val` method in `cli.rs`, which relies on
   the types defined in the `data/` source directory.
3. Performance: originally, I implemented this very lazily without paying any
   attention to performance (I was using Rust for its type system, not for
   performance). Then it turned out Ryan and Zach's javascript implementation
   was faster than my Rust implementation because I was just cloning everywhere
   so I didn't have to worry about the borrow checker (which isn't always
   friendly to recursive types), a common theme in lazy Rust code. But
   javascript code being faster than Rust code was obviously unacceptable to my
   Rust-obsessed brain, so I did a bunch of performance optimization, and now
   this code is quite fast IMO. That's mostly in the reduction code and is
   documented there. Interestingly, the remaining runtime (about 2/3 of the
   typical runtime of the program) is in the `get_fresh_ident` method, which
   has to do a bunch of messy stuff with owned strings.

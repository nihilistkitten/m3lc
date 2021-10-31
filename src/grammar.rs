//! The abstract grammar.
//!
//! # Implementation
//! Note that we generally use owned strings. You can probably implement this with `&str`s, but I
//! didn't think the added complexity would be worth it; this code is not particularly
//! performance-sensitive.

use std::fmt::Display;

/// A single lambda term:
#[derive(Debug, PartialEq)]
pub enum Term {
    /// A named variable.
    Var(String),

    /// A lambda abstraction.
    Lam { param: String, rule: Box<Term> },

    /// A function application.
    Appl { left: Box<Term>, right: Box<Term> },
}

impl From<String> for Term {
    fn from(s: String) -> Self {
        Self::Var(s)
    }
}

impl From<String> for Box<Term> {
    fn from(s: String) -> Self {
        Self::new(s.into())
    }
}

impl From<&str> for Term {
    fn from(s: &str) -> Self {
        Self::Var(s.into())
    }
}

impl From<&str> for Box<Term> {
    fn from(s: &str) -> Self {
        // type inference isn't good enough to chain two intos here
        Self::new(s.into())
    }
}

impl Display for Term {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::Var(s) => s.to_string(),
            Self::Lam { param, rule } => format!("fn {} => {}", param, rule),
            Self::Appl { left, right } => {
                // we parenthesize lams (for readability) and right-heavy appls (for associativity)
                let left_fmt = if let Self::Lam { .. } = left.as_ref() {
                    format!("({})", left)
                } else {
                    format!("{}", left)
                };
                let right_fmt = if let Self::Var(_) = right.as_ref() {
                    format!("{}", right)
                } else {
                    format!("({})", right)
                };
                format!("{} {}", left_fmt, right_fmt)
            }
        };
        write!(f, "{}", message)
    }
}

/// A named lambda term, for later substitution.
#[derive(Debug, PartialEq)]
pub struct Defn {
    name: String,
    term: Term,
}

impl Display for Defn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} := {}", self.name, self.term)
    }
}

/// A file of defns, with a main term.
pub struct File {
    defns: Vec<Defn>,
    main: Term,
}

impl Display for File {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for defn in &self.defns {
            writeln!(f, "{}", defn)?;
        }
        write!(f, "main := {}", self.main)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use Term::{Appl, Lam, Var};

    macro_rules! term_display_tests { ($($name:ident: $expected:expr, $ast:expr)*)  => {
    mod term_display {
        use super::*;

        $(
        #[test]
        fn $name() {
            assert_eq!(format!("{}", $ast), $expected);
        }

        )*
    }}}

    term_display_tests! {
        identifier: "s", Var("s".into())
        identity: "fn x => x", Lam{param: "x".into(), rule: "x".into()}
        one: "fn f => fn a => f a", Lam{
            param: "f".into(),
            rule: Lam{
                param: "a".into(),
                rule: Appl{
                    left: "f".into(),
                    right: "a".into()
                }.into()
            }.into()
        }
        succ: "fn n => fn f => fn a => f (n f a)", Lam{
            param: "n".into(),
            rule: Lam{
                param: "f".into(),
                rule: Lam{
                    param: "a".into(),
                    rule: Appl{
                        left: "f".into(),
                        right: Appl{
                            left: Appl{
                                left: "n".into(),
                                right: "f".into()
                            }.into(),
                            right: "a".into()
                        }.into()
                    }.into()
                }.into()
            }.into()
        }
        yc: "fn g => (fn x => g (x x)) (fn x => g (x x))", Lam{
            param: "g".into(),
            rule: Appl{
                left: Lam {
                    param: "x".into(),
                    rule: Appl {
                        left: "g".into(),
                        right: Appl {
                            left: "x".into(),
                            right: "x".into()
                        }.into()
                    }.into()
                }.into(),
                right: Lam {
                    param: "x".into(),
                    rule: Appl {
                        left: "g".into(),
                        right: Appl {
                            left: "x".into(),
                            right: "x".into()
                        }.into()
                    }.into()
                }.into()
            }.into()
        }
    }

    #[test]
    fn defn_display() {
        let defn = Defn {
            name: "ident".into(),
            term: Lam {
                param: "x".into(),
                rule: "x".into(),
            },
        };
        assert_eq!(format!("{}", defn), "ident := fn x => x");
    }

    #[test]
    fn file_display() {
        let defns = vec![
            Defn {
                name: "ident".into(),
                term: Lam {
                    param: "x".into(),
                    rule: "x".into(),
                },
            },
            Defn {
                name: "zero".into(),
                term: Lam {
                    param: "f".into(),
                    rule: Lam {
                        param: "a".into(),
                        rule: "a".into(),
                    }
                    .into(),
                },
            },
        ];
        let main = Appl {
            left: "ident".into(),
            right: "zero".into(),
        };
        let file = File { defns, main };
        let expected = "\
            ident := fn x => x\n\
            zero := fn f => fn a => a\n\
            main := ident zero\
        ";
        assert_eq!(format!("{}", file), expected);
    }
}

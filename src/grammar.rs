//! The abstract grammar.
//!
//! # Implementation
//!
//! Everything is heap-allocated. This isn't great for performance. You obviously have to box the
//! recursive types so the compiler can size the type, but it makes for awkward code (lots of
//! `into`s to coerce to Box/String).
//!
//! More of a choice is in using owned Strings. You can probably implement this with `&str`s, but I
//! didn't think the added complexity would be worth it; this code is not particularly
//! performance-sensitive, and the `into`s aren't _that_ awkward.

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

impl Defn {
    /// Create a new `Defn`.
    pub const fn new(name: String, term: Term) -> Self {
        Self { name, term }
    }

    /// Get a reference to the defn's name.
    pub fn name(&self) -> &str {
        self.name.as_ref()
    }

    /// Get a reference to the defn's term.
    pub const fn term(&self) -> &Term {
        &self.term
    }
}

impl Display for Defn {
    // Displaying `defn` does not include the closing ;, because a) that's how it's implemented in
    // the grammar, and b) I think it looks better that way.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} := {}", self.name, self.term)
    }
}

/// A file of defns, with a main term.
#[derive(Debug, PartialEq)]
pub struct File {
    defns: Vec<Defn>,
    main: Term,
}

impl File {
    /// Create a new `File`.
    pub const fn new(defns: Vec<Defn>, main: Term) -> Self {
        Self { defns, main }
    }

    /// Get a reference to the file's defns.
    pub fn defns(&self) -> &[Defn] {
        self.defns.as_ref()
    }

    /// Get a reference to the file's main.
    pub const fn main(&self) -> &Term {
        &self.main
    }

    /// Unroll the file into a single lambda.
    ///
    /// We think of main as abstracted over each defn in reverse, i.e.
    /// ```m3lc
    /// foo := term1
    /// bar := term2
    /// main := term3
    /// ```
    ///
    /// unrolls into a `Term` equivalent to
    /// ```m3lc
    /// (fn foo => (fn bar => term3) term2) term1
    /// ```
    pub fn unroll(mut self) -> Term {
        for defn in self.defns.into_iter().rev() {
            self.main = Term::Appl {
                left: Term::Lam {
                    param: defn.name,
                    rule: self.main.into(),
                }
                .into(),
                right: defn.term.into(),
            }
        }
        self.main
    }
}

impl Display for File {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for defn in &self.defns {
            writeln!(f, "{};", defn)?;
        }
        write!(f, "main := {};", self.main)
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
            ident := fn x => x;\n\
            zero := fn f => fn a => a;\n\
            main := ident zero;\
        ";
        assert_eq!(format!("{}", file), expected);
    }

    #[test]
    fn test_unroll() {
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
        let input = File { defns, main };
        let expected = Appl {
            left: Lam {
                param: "ident".into(),
                rule: Appl {
                    left: Lam {
                        param: "zero".into(),
                        rule: Appl {
                            left: "ident".into(),
                            right: "zero".into(),
                        }
                        .into(),
                    }
                    .into(),
                    right: Lam {
                        param: "f".into(),
                        rule: Lam {
                            param: "a".into(),
                            rule: "a".into(),
                        }
                        .into(),
                    }
                    .into(),
                }
                .into(),
            }
            .into(),
            right: Lam {
                param: "x".into(),
                rule: "x".into(),
            }
            .into(),
        };
        assert_eq!(input.unroll(), expected);
    }
}
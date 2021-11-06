//! The abstract grammar.
use std::fmt::Display;

/// A single lambda term.
#[derive(Clone, Debug, PartialEq)]
pub enum Term {
    // Many things here are heap-allocated. You obviously have to box the recursive types so the
    // compiler can size the type, but it makes for awkward code (lots of `into`s to coerce to
    // Box/String).
    //
    // More of a choice is in using owned Strings. You can probably implement this with `&str`s, but I
    // didn't think the added complexity would be worth it; this code is not particularly
    // performance-sensitive, and the `into`s aren't _that_ awkward. The big issue with using borrows
    // is in the `reduce::get_fresh_ident` function, which requires mutability. There is an
    // explanation of why this is a problem in that function. A second concern is that in a
    // hypothetical REPL, the &str would only live to the end of the loop, we'd want it to live for
    // the duration of the REPL so that we could reference terms in other terms.
    //
    /// A named variable.
    Var(String),

    /// A lambda abstraction.
    Lam { param: String, rule: Box<Term> },

    /// A function application.
    Appl { left: Box<Term>, right: Box<Term> },
}

// Importantly, this impl converts a string into a `Term::Var`, it does _not_ try to parse the string
// as a lambda. This would be fallible behavior, which is not ok for `From`.
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
        s.to_string().into()
    }
}

impl From<&str> for Box<Term> {
    fn from(s: &str) -> Self {
        // Type inference is not good enough to chain two intos here; it in particular can't get
        // that `Term` is the intermediate type.
        Self::new(s.into())
    }
}

impl Display for Term {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::Var(s) => s.to_string(),
            Self::Lam { param, rule } => format!("fn {} => {}", param, rule),

            // We need special handling here to deal with parenthesization. I _think_ that this
            // parenthesization is invertible, i.e. that we don't drop any associativity
            // information and so `to_term(t.to_string())` always produces the original term.
            // But I haven't verified this formally or anything. My informal analysis is explained
            // in the comments below.
            Self::Appl { left, right } => {
                let left_fmt = if let Self::Lam { .. } = left.as_ref() {
                    // parenthesize lambdas on the left: consider `(fn x => x) g` vs `fn x => x g`
                    format!("({})", left)
                } else {
                    // no need to parenthesize vars, ever
                    //
                    // no need to parenthesize left-heavy appls because of associativity
                    left.to_string()
                };
                let right_fmt = if let Self::Var(_) = right.as_ref() {
                    // no need to parenthesize vars, ever
                    right.to_string()
                } else {
                    // parenthesize appls on the right: consider `x y z` vs `x (y z)`
                    //
                    // no need to parenthesize lambdas on the right: `fn` sort of does this for us,
                    // but we do it anyway for readability: consider
                    // `(fn x => xx) fn x => xx` vs `(fn x => xx) (fn x => xx)`
                    format!("({})", right)
                };
                left_fmt + " " + &right_fmt
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
    #[must_use]
    pub const fn new(name: String, term: Term) -> Self {
        Self { name, term }
    }

    /// Get a reference to the defn's name.
    #[must_use]
    pub fn name(&self) -> &str {
        self.name.as_ref()
    }

    /// Get a reference to the defn's term.
    #[must_use]
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
    #[must_use]
    pub const fn new(defns: Vec<Defn>, main: Term) -> Self {
        Self { defns, main }
    }

    /// Get a reference to the file's defns.
    #[must_use]
    pub fn defns(&self) -> &[Defn] {
        self.defns.as_ref()
    }

    /// Get a reference to the file's main.
    #[must_use]
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
    #[must_use]
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

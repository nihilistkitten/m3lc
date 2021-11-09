//! The Church numerals.
use lazy_static::lazy_static;

use crate::grammar::Term;
use Term::{Appl, Lam, Var};

lazy_static! {
    static ref SUCC: Term = Lam {
        param: "n".into(),
        rule: Lam {
            param: "f".into(),
            rule: Lam {
                param: "a".into(),
                rule: Appl {
                    left: "f".into(),
                    right: Appl {
                        left: Appl {
                            left: "n".into(),
                            right: "f".into(),
                        }
                        .into(),
                        right: "a".into()
                    }
                    .into()
                }
                .into()
            }
            .into()
        }
        .into()
    };
}

impl Term {
    /// Compute the successor of n.
    ///
    /// # Example
    /// ```
    /// # use m3lc::Term;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// #
    /// let three: Term = 3.into();
    /// assert!(three.succ().alpha_equiv(&4.into()));
    /// #
    /// # Ok(())}
    /// ```
    #[must_use]
    pub fn succ(self) -> Self {
        Appl {
            left: SUCC.clone().into(),
            right: self.into(),
        }
        .reduce(false)
    }
}

impl From<usize> for Term {
    fn from(n: usize) -> Self {
        // Imperative instead of recursive to avoid repeated clones of `SUCC` and so we can use
        // this to test `succ`.
        let mut out: Self = "a".into();
        for _ in 0..n {
            out = Appl {
                left: "f".into(),
                right: out.into(),
            };
        }
        Lam {
            param: "f".into(),
            rule: Lam {
                param: "a".into(),
                rule: out.into(),
            }
            .into(),
        }
    }
}

/// The `Term` is not a Church numeral.
#[derive(Debug)]
pub struct NotChurchNum;

impl TryFrom<&Term> for usize {
    type Error = NotChurchNum;

    fn try_from(term: &Term) -> Result<Self, Self::Error> {
        if let Lam { param, rule } = term {
            let f = param; // the f in fn f => fn a => f (f (... a))
            if let Lam { param, rule } = rule.as_ref() {
                let mut curr = rule.as_ref(); // the current step in the iteration
                let a = param; // the a in the above

                // We're looking for a right-heavy binary tree of `Appl`s, where each leaf is a
                // `Var(f)`, except for a `Var(a)` at the very bottom. We're going to iteratively
                // traverse down this tree, always checking the leaf on the left, and then when we
                // stop hitting `Appl`s, we should hit `Var(a)`. All the while, we keep a count of
                // the number of `f`s that we've hit.
                let mut n = 0;
                while let Appl { left, right } = curr {
                    // check that the left is a Var(f)
                    if matches!(left.as_ref(), Var(x) if x == f) {
                        n += 1;
                        curr = right;
                    } else {
                        return Err(NotChurchNum);
                    }
                }

                // We stopped hitting `Appl`s, so we should have a `Var(a)`.
                if matches!(curr, Var(x) if x == a) {
                    return Ok(n);
                }
            }
        }
        Err(NotChurchNum)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod from_usize {
        use super::*;

        #[test]
        fn zero() {
            let got: Term = 0.into();
            let expected = Lam {
                param: "f".into(),
                rule: Lam {
                    param: "a".into(),
                    rule: "a".into(),
                }
                .into(),
            };

            assert_eq!(got, expected);
        }

        #[test]
        fn three() {
            let got: Term = 3.into();
            let expected = Lam {
                param: "f".into(),
                rule: Lam {
                    param: "a".into(),
                    rule: Appl {
                        left: "f".into(),
                        right: Appl {
                            left: "f".into(),
                            right: Appl {
                                left: "f".into(),
                                right: "a".into(),
                            }
                            .into(),
                        }
                        .into(),
                    }
                    .into(),
                }
                .into(),
            };

            assert_eq!(got, expected);
        }
    }

    mod succ {
        use super::*;

        #[test]
        fn zero() {
            let zero: Term = 0.into();
            let one = Lam {
                param: "f".into(),
                rule: Lam {
                    param: "a".into(),
                    rule: Appl {
                        left: "f".into(),
                        right: "a".into(),
                    }
                    .into(),
                }
                .into(),
            };
            assert!(zero.succ().alpha_equiv(&one));
        }

        #[test]
        fn seventeen() {
            let seventeen: Term = 17.into();
            assert!(seventeen.succ().alpha_equiv(&18.into()));
        }
    }

    mod try_into_usize {
        use super::*;

        /// simple conversion from num to term to num
        macro_rules! try_into_usize_nums { ($($name:ident: $input:expr)*) => {
            $(
            #[test]
            fn $name() -> Result<(), NotChurchNum> {
                let $name: Term = $input.into();
                let got: usize = (&$name).try_into()?;
                assert_eq!(got, $input);
                Ok(())
            }
            )*
        }}

        try_into_usize_nums! {
            zero: 0
            one: 1
            two: 2
            three: 3
            seventeen: 17
            one_forty_three: 143
        }

        /// for more complicated terms that can't be constructed as num.into()
        macro_rules! try_into_usize_oks { ($($name:ident: $expected: expr, $ast:expr)*) => {
            $(
            #[test]
            fn $name() -> Result<(), NotChurchNum> {
                let got: usize = (&$ast).try_into()?;
                assert_eq!(got, $expected);
                Ok(())
            }
            )*
        }}

        try_into_usize_oks! {
            one_weird_names: 1, Lam {
                param: "f.1234".into(),
                rule: Lam {
                    param: "qwerty".into(),
                    rule: Appl {
                        left: "f.1234".into(),
                        right: "qwerty".into()
                    }.into()

                }.into()
            }
            three_weird_names: 3, Lam {
                param: "per.3141".into(),
                rule: Lam {
                    param: "aspera.5965".into(),
                    rule: Appl {
                        left: "per.3141".into(),
                        right: Appl {
                            left: "per.3141".into(),
                            right: Appl {
                                left: "per.3141".into(),
                                right: "aspera.5965".into(),
                            }
                            .into(),
                        }
                        .into(),
                    }
                    .into(),
                }
                .into(),
            }
        }

        macro_rules! try_into_usize_errs { ($($name:ident: $ast:expr)*) => {
            $(
            #[test]
            fn $name() {
                let got: Result<usize, _> = (&$ast).try_into();
                assert!(got.is_err());
            }
            )*
        }}

        try_into_usize_errs! {
            inconsistent_names: Lam {
                param: "g".into(),
                rule: Lam {
                    param: "a".into(),
                    rule: Appl {
                        left: "g".into(),
                        right: Appl {
                            left: "f".into(),
                            right: "a".into(),
                        }
                        .into(),
                    }
                    .into(),
                }
                .into(),
            }
            identity: Lam {
                param: "x".into(),
                rule: "x".into()
            }
        }
    }
}

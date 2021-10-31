//! Normal-order beta reduction of lambda terms.
use std::cell::RefCell;

use crate::grammar::Term;

impl Term {
    /// Check term equivalence under alpha-renaming.
    ///
    /// The idea is to maintain a context which stores the existing lambda abstractions, _in
    /// order_. This context essentially associates variables from each term. We can therefore use
    /// this to check equivalence whenever we see a `Var`.
    ///
    /// We don't want to use `subst` here because a big motivation for implementing this function
    /// is to enable testing `subst` without relying on implementation details of `get_fresh`.
    fn alpha_equiv(&self, other: &Self) -> bool {
        self.alpha_equiv_impl(other, &mut vec![])
    }

    fn alpha_equiv_impl<'a>(&'a self, other: &'a Self, ctx: &mut Vec<(&'a str, &'a str)>) -> bool {
        match (self, other) {
            // handling var: if x and y are bound in the same lambda, return true
            (Self::Var(x), Self::Var(y)) => {
                x == y
                    || ctx
                        .iter()
                        .rfind(|(a, b)| a == x || b == y) // find the most recent binding of x or y
                        .map_or(false, |(a, b)| a == x && b == y)
            } // it should also bind the other

            // handling lam: store params in the ctx and recurse on the rules
            (
                Self::Lam {
                    param: param1,
                    rule: rule1,
                },
                Self::Lam {
                    param: param2,
                    rule: rule2,
                },
            ) => {
                // I think there ought to be a better way to handle this with iterator adapters,
                // but this works for now. Certainly it's not pretty code, but this is an ugly
                // procedure by nature.

                let out = rule1.alpha_equiv_impl(rule2, {
                    // push the new params onto the ctx
                    ctx.push((param1, param2));
                    ctx
                });
                // pop the params after the check is done
                let _ = ctx.pop();
                out
            }

            // handling appl: recurse on both sides
            (
                Self::Appl {
                    left: left1,
                    right: right1,
                },
                Self::Appl {
                    left: left2,
                    right: right2,
                },
            ) => left1.alpha_equiv_impl(left2, ctx) && right1.alpha_equiv_impl(right2, ctx),

            // other cases: just return false; even aside from substitution, they have different
            // term structures
            _ => false,
        }
    }

    /// Perform substitution of `replace` for `with` in `self`.
    ///
    /// [s/x] x           := s
    /// [s/x] y           := y
    /// [s/x] (fn x => t) := (fn x => t)
    /// [s/x] (fn y => t) := (fn z => [s/x] ([z/y] t)) for fresh z
    /// [s/x] (t1 t2)     := ([s/x] t1) ([s/x] t2)
    ///
    fn subst(self, replace: &str, with: Self) -> Self {
        match self {
            Self::Var(ref s) => {
                if s == replace {
                    with
                } else {
                    self
                }
            }
            Self::Lam { param, rule } => {
                if param == replace {
                    // can't use "self" here because we move `rule` for handling the else case
                    Self::Lam { param, rule }
                } else {
                    let new_var = get_fresh(&param);
                    Self::Lam {
                        param: new_var.clone(), // we need to clone the String that get_fresh gives us
                        rule: rule
                            .subst(&param, new_var.into())
                            .subst(replace, with)
                            .into(),
                    }
                }
            }
            Self::Appl { left, right } => Self::Appl {
                left: left.subst(replace, with.clone()).into(),
                right: right.subst(replace, with).into(),
            },
        }
    }
}

// global mutable state shouldn't be shared across threads (and so rust needs us to do this)
thread_local!(static COUNTER: RefCell<usize> = 0.into());

/// Generate a fresh variable name.
///
/// The grammar forbids variable names containing "." and the global counter ensures that specific
/// name hasn't been generated yet.
fn get_fresh(s: &str) -> String {
    COUNTER.with(|c| {
        *c.borrow_mut() += 1;
        s.to_string() + "." + &c.borrow().to_string()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    mod get_fresh {
        use super::*;
        use std::collections::HashSet;

        #[test]
        fn foo() {
            let mut uniq = HashSet::new();
            assert!((0..100).map(|_| get_fresh("foo")).all(|x| uniq.insert(x)));
        }

        #[test]
        fn mixed() {
            let mut uniq = HashSet::new();
            assert!([
                "hello",
                "goodbye",
                "foo",
                "bar",
                "foo",
                "goodbye",
                "World",
                "x",
                "y",
                "foo",
                "foo_world"
            ]
            .into_iter()
            .map(get_fresh)
            .all(|x| uniq.insert(x)));
        }
    }

    mod alpha_equiv {
        use super::*;
        use Term::{Appl, Lam};
        // takes a name, two asts, and a bool
        #[test]
        fn identical() {
            assert!(Lam {
                param: "x".into(),
                rule: "x".into()
            }
            .alpha_equiv(&Lam {
                param: "x".into(),
                rule: "x".into()
            }));
        }

        #[test]
        fn simple_lam() {
            assert!(Lam {
                param: "x".into(),
                rule: "x".into()
            }
            .alpha_equiv(&Lam {
                param: "y".into(),
                rule: "y".into()
            }));
        }

        #[test]
        fn free_vars() {
            // the different free variables mean these shouldn't be alpha-equivalent
            assert!(!Lam {
                param: "x".into(),
                rule: Appl {
                    left: "x".into(),
                    right: "y".into()
                }
                .into()
            }
            .alpha_equiv(&Lam {
                param: "x".into(),
                rule: Appl {
                    left: "x".into(),
                    right: "z".into()
                }
                .into()
            }));
        }

        #[test]
        fn different_structure() {
            assert!(!Lam {
                param: "x".into(),
                rule: "y".into()
            }
            .alpha_equiv(&Appl {
                left: "x".into(),
                right: "y".into()
            }));
        }

        #[test]
        fn nested_lam() {
            assert!(Lam {
                param: "x".into(),
                rule: Lam {
                    param: "y".into(),
                    rule: "z".into(),
                }
                .into()
            }
            .alpha_equiv(&Lam {
                param: "x".into(),
                rule: Lam {
                    param: "y".into(),
                    rule: "z".into(),
                }
                .into()
            }));
        }

        #[test]
        fn nested_lam_different_names() {
            assert!(Lam {
                param: "x".into(),
                rule: Lam {
                    param: "y".into(),
                    rule: "z".into(),
                }
                .into()
            }
            .alpha_equiv(&Lam {
                param: "a".into(),
                rule: Lam {
                    param: "b".into(),
                    rule: "z".into(),
                }
                .into()
            }));
        }
    }

    mod subst {
        use super::*;
        use Term::{Appl, Lam};

        #[test]
        fn shadowing() {
            let init = Lam {
                param: "z".into(),
                rule: "x".into(),
            };
            let term = Lam {
                param: "x".into(),
                rule: "y".into(),
            };

            let out = term.subst("y", init);
            let expected = Lam {
                param: "z".into(),
                rule: Lam {
                    param: "y".into(),
                    rule: "x".into(), // this name is free in `init`, so should be preserved
                }
                .into(),
            };

            assert!(out.alpha_equiv(&expected));
        }

        #[test]
        fn no_sub() {
            let init = Appl {
                left: "x".into(),
                right: "y".into(),
            };
            let term = Lam {
                param: "x".into(),
                rule: "y".into(),
            };

            let out = term.clone().subst("z", init); // z not in FV(term), so no sub necessary
            assert!(term.alpha_equiv(&out));
        }
    }
}
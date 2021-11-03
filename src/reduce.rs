//! Normal-order beta reduction of lambda terms.
use std::cell::RefCell;

use crate::grammar::Term;

impl Term {
    /// Perform normal-order beta reduction.
    ///
    /// # Safety
    /// The halting problem is a thing. Ergo, this can cause unhandled infinite regress.
    pub fn reduce(self) -> Self {
        match self {
            // Vars are irreducible.
            Self::Var(_) => self,

            //           t ~~> t'
            // ----------------------------
            // (fn x => t) ~~> (fn x => t')
            Self::Lam { param, rule } => Self::Lam {
                param,
                rule: rule.reduce().into(),
            },

            // Handle appl.
            Self::Appl { left, right } => {
                if let Self::Lam { param, rule } = *left {
                    // -------------------------
                    // (fn x => t) s ~~> [s/x] t
                    rule.subst(&param, *right).reduce()
                } else if left.is_irreducible() {
                    // Only reduce the right if the left is already reduced.
                    //
                    // t1 irr    t2 ~~> t2'
                    // ----------------------
                    //  (t1 t2) ~~> (t1 t2')
                    //
                    // We need a special case for the left being irreducible, instead of just
                    // reducing on the left, then outside (in case a lambda was created), then the
                    // right, because after reducing an outer lambda created by reducing on the
                    // left, we could need to again reduce on the left, because that outer
                    // reduction could have created a reducible left side.
                    Self::Appl {
                        left,
                        right: right.reduce().into(),
                    }
                } else {
                    // Note that here left is not a Var, because it's reducible, and not a Lam,
                    // because that was checked earlier. Therefore left is (t1 t2) and reducible,
                    // and so one of two rules applies:
                    //
                    //          t1 ~~> t1'
                    // ------------------------------
                    // ((t1 t2) t3) ~~> ((t1' t2) t3)
                    //
                    //     t1 irr      t2 ~~> t2'
                    // ------------------------------
                    // ((t1 t2) t3) ~~> ((t1 t2') t3)
                    //
                    // Either way, we can implement the reduction by recurring to the left.
                    Self::Appl {
                        left: left.reduce().into(),
                        right,
                    }
                    // It's important to reduce the whole thing again, in case the reduction turned
                    // left into a lambda.
                    .reduce()
                }
            }
        }
    }

    /// Check whether the term is beta-reducible.
    fn is_irreducible(&self) -> bool {
        match self {
            // -----
            // x irr
            Self::Var(_) => true,

            Self::Appl { left, right } => {
                if let Self::Lam { .. } = left.as_ref() {
                    // Lams applied to terms are always reducible.
                    false
                } else {
                    // Follows from one of these rules, depending on the variant of left:
                    //
                    //  (t1 t2) irr    t3 irr
                    // ----------------------
                    //    ((t1 t2) t3) irr
                    //
                    //   t irr
                    // ---------
                    // (x t) irr
                    left.is_irreducible() && right.is_irreducible()
                }
            }

            //      t irr
            // ---------------
            // (fn x => t) irr
            Self::Lam { rule, .. } => rule.is_irreducible(),
        }
    }

    /// Perform substitution of `replace` for `with` in `self`.
    fn subst(self, replace: &str, with: Self) -> Self {
        match self {
            // [s/x] x := s
            Self::Var(ref s) if s == replace => with,

            // [s/x] y := y
            Self::Var(_) => self,

            // [s/x] (fn x => t) := (fn x => t)
            Self::Lam { ref param, .. } if param == replace => self,

            // [s/x] (fn y => t) := (fn z => [s/x] ([z/y] t)) for fresh z
            Self::Lam { param, rule } => {
                let new_var = get_fresh_ident(&param);
                Self::Lam {
                    param: new_var.clone(), // we need new_var for the param and the recursive subst
                    rule: rule
                        .subst(&param, new_var.into())
                        .subst(replace, with)
                        .into(),
                }
            }

            // [s/x] (t1 t2) := ([s/x] t1) ([s/x] t2)
            Self::Appl { left, right } => Self::Appl {
                left: left.subst(replace, with.clone()).into(),
                right: right.subst(replace, with).into(),
            },
        }
    }

    /// Check term equivalence under alpha-renaming.
    pub fn alpha_equiv(&self, other: &Self) -> bool {
        self.alpha_equiv_impl(other, &mut vec![])
    }

    fn alpha_equiv_impl<'a>(&'a self, other: &'a Self, ctx: &mut Vec<(&'a str, &'a str)>) -> bool {
        // The idea is to maintain a context which stores the existing lambda abstractions, _in
        // order_. This context essentially associates variables from each term. We can therefore use
        // this to check equivalence whenever we see a `Var`.
        //
        // We don't want to use `subst` here because a big motivation for implementing this function
        // is to enable testing `subst` without relying on implementation details of `get_fresh`.
        match (self, other) {
            // handling var: if x and y are most recently bound in the same lambda, return true
            (Self::Var(x), Self::Var(y)) => {
                #[allow(clippy::map_unwrap_or)] // slight performance tradeoff, but more readable
                ctx.iter()
                    .rfind(|(a, b)| a == x || b == y) // find the most recent binding of x or y
                    .map(|(a, b)| a == x && b == y) // it should also bind the other
                    .unwrap_or(x == y) // if neither is bound, they should be equal
            }

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
                // Push the new binding onto the context, compare the rules, then pop it off the
                // context so that parent calls don't inherit our binding.
                ctx.push((param1, param2));
                let out = rule1.alpha_equiv_impl(rule2, ctx);
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
}

// global mutable state shouldn't be shared across threads (and so rust needs us to do this)
thread_local!(static COUNTER: RefCell<usize> = 0.into());

/// Generate a fresh variable name.
fn get_fresh_ident(s: &str) -> String {
    // The grammar forbids variable names containing ".", so this name can't have been written by
    // the user, and the global counter ensures that specific name hasn't been generated yet by
    // this method, which is the only way new names get added to the AST.
    //
    // This function is the primary reason we store owned Strings in AST Terms, instead of borrowed
    // `&str`s. We need to be able to append onto the end of `s`, but `&str`s can't guarantee (and
    // obviously in general it's highly unlikely) that the referenced string will be next to the
    // string we're appending to the end. Returning a `String` from this function doesn't work if
    // `Term` expects a `&str`, because the reference won't live past the end of `Term::reduce`.
    COUNTER.with(|c| {
        *c.borrow_mut() += 1;
        s.split('.')
            .next()
            .expect("split gives at least one item")
            .to_string()
            + "."
            + &c.borrow().to_string()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use Term::{Appl, Lam, Var};

    mod reduction {
        use super::*;
        use crate::{to_term, ParserResult};

        #[test]
        /// Test reducing a var.
        fn var() {
            assert_eq!(Var("x".into()).reduce(), "x".into());
        }

        #[test]
        /// Test a simple lambda application, (fn x => x) z.
        fn simple_lam_appl() {
            let input = Appl {
                left: Lam {
                    param: "x".into(),
                    rule: "x".into(),
                }
                .into(),
                right: "z".into(),
            };

            assert_eq!(input.reduce(), "z".into());
        }

        #[test]
        /// Test a const lambda appl'd to something.
        fn lam_appl_const() {
            let input = Appl {
                left: Lam {
                    param: "x".into(),
                    rule: "y".into(),
                }
                .into(),
                right: "z".into(),
            };

            assert_eq!(input.reduce(), "y".into());
        }

        #[test]
        /// Test a lambda applied to another lambda.
        fn lam_app_lam() {
            let input = Appl {
                left: Appl {
                    left: Lam {
                        param: "x".into(),
                        rule: "x".into(),
                    }
                    .into(),
                    right: Lam {
                        param: "x".into(),
                        rule: "x".into(),
                    }
                    .into(),
                }
                .into(),
                right: "a".into(),
            };
            assert_eq!(input.reduce(), "a".into());
        }

        // takes a name, a string representing the term to be reduced, and a string representing
        // the expected normal form
        macro_rules! beta_reduction_tests { ($($name:ident: $input:expr, $expected:expr)*) => {
            $(
            #[test]
            fn $name() -> ParserResult<()> {
                // This is not a proper unit test because of the dependency on `to_term`, but it
                // makes tests _much_ easier to develop.
                assert!(to_term($input)?.reduce().alpha_equiv(&to_term($expected)?));
                Ok(())
            }
            )*
        }}

        beta_reduction_tests! {
            nested_sub: "(fn f => fn a => f) x", "fn a => x"
            order_matters: "(fn f => fn a => f (f a)) (fn q => r) a b", "r b"
            many_renames: "(fn f => fn y => fn x => x (y f)) y x f", "f (x y)"
        }
    }

    mod is_irreducible {
        use super::*;

        #[test]
        fn var() {
            assert!(Var("x".into()).is_irreducible());
        }

        #[test]
        fn lam() {
            assert!(Lam {
                param: "x".into(),
                rule: "y".into()
            }
            .is_irreducible());
        }

        #[test]
        fn lam_reducible_rule() {
            assert!(!Lam {
                param: "x".into(),
                rule: Appl {
                    left: Lam {
                        param: "x".into(),
                        rule: "x".into(),
                    }
                    .into(),
                    right: "z".into()
                }
                .into()
            }
            .is_irreducible());
        }

        #[test]
        fn lam_appl() {
            assert!(!Appl {
                left: Lam {
                    param: "x".into(),
                    rule: "x".into(),
                }
                .into(),
                right: "z".into()
            }
            .is_irreducible());
        }

        #[test]
        /// Test a lambda applied to another lambda.
        fn lam_app_lam() {
            assert!(!Appl {
                left: Appl {
                    left: Lam {
                        param: "x".into(),
                        rule: "x".into(),
                    }
                    .into(),
                    right: Lam {
                        param: "x".into(),
                        rule: "x".into(),
                    }
                    .into(),
                }
                .into(),
                right: "a".into(),
            }
            .is_irreducible());
        }
    }

    mod get_fresh_ident {
        use super::*;
        use std::collections::HashSet;

        #[test]
        fn foo() {
            let mut uniq = HashSet::new();
            assert!((0..100)
                .map(|_| get_fresh_ident("foo"))
                .all(|x| uniq.insert(x)));
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
            .map(get_fresh_ident)
            .all(|x| uniq.insert(x)));
        }
    }

    mod alpha_equiv {
        use super::*;

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
        /// Test `alpha_equiv` for lambdas with the same argument and different binding.
        ///
        /// This used to cause a bug, because for Vars we checked equivalence of the identifier
        /// before looking up the identifier in the context. We only care about the actual
        /// identifier if _neither_ identifier is bound in the context.
        fn different_binding() {
            assert!(!Lam {
                param: "x".into(),
                rule: "x".into()
            }
            .alpha_equiv(&Lam {
                param: "y".into(),
                rule: "x".into()
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

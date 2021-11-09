//! Normal-order beta reduction of lambda terms.
use std::{cell::RefCell, mem};

use crate::grammar::Term;

impl Term {
    /// Perform normal-order beta reduction.
    ///
    /// # Safety
    /// The halting problem is a thing. Ergo, this can cause unhandled infinite regress.
    #[must_use]
    pub fn reduce(mut self, verbose: bool) -> Self {
        while !self.is_irreducible() {
            if verbose {
                println!("{}", self);
            }
            self.reduction_step();
        }
        self
    }

    fn reduction_step(&mut self) {
        match self {
            // If we get here, then there's a bug and reduce will loop infinitely, so better to
            // fail fast.
            Self::Var(_) => unreachable!("vars are irreducible"),

            //           t ~~> t'
            // ----------------------------
            // (fn x => t) ~~> (fn x => t')
            Self::Lam { rule, .. } => rule.reduction_step(),

            Self::Appl { left, right } => {
                if let Self::Lam { .. } = left.as_mut() {
                    // -------------------------
                    // (fn x => t) s ~~> [s/x] t
                    //
                    // We have a special method here, `apply`, which does some performance hacks on
                    // top of `subst` to avoid unnecessary clones. That's documented in the body of
                    // that method.
                    self.apply();
                } else if left.is_irreducible() {
                    // t1 irr    t2 ~~> t2'
                    // ----------------------
                    //  (t1 t2) ~~> (t1 t2')
                    right.reduction_step();
                } else {
                    //          t1 ~~> t1'
                    // ------------------------------
                    // ((t1 t2) t3) ~~> ((t1' t2) t3)
                    //
                    //     t1 irr      t2 ~~> t2'
                    // ------------------------------
                    // ((t1 t2) t3) ~~> ((t1 t2') t3)
                    left.reduction_step();
                }
            }
        }
    }

    /// Given an appl with a lam on the left, apply the left to the right.
    fn apply(&mut self) {
        // Put a placeholder into self so we get ownership of the dereferenced value. Note that
        // empty strings don't allocate.
        let to_apply = mem::replace(self, Self::Var(String::new()));

        // We have to traverse down the struct to get to the lambda on the left. This is guaranteed
        // to be ok, because `apply` can only be called when we've matched exactly this pattern
        // already.
        if let Self::Appl {
            left: box Self::Lam { param, mut rule },
            right,
        } = to_apply
        {
            (*rule).subst(&param, &*right);
            // Now we can write `rule` into the memory of `self` (currently occupied by the
            // placeholder `Var("")`). If we hadn't done the `mem::replace" trick, this would
            // break borrow rules, because it would require a mutable reference to `self` and a
            // reference to `right` (which `rule` depends on). So unless we wanted to use
            // `unsafe`, we'd either have to clone `right` or clone `rule`.
            *self = *rule;
        } else {
            unreachable!("apply only called with appl with lam on left");
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
    fn subst<T>(&mut self, replace: &str, with: &T)
    where
        // Into<Self> so we can pass &strs, so we don't have to clone new_var until needed.
        // Refs so we can wait to clone until we need to. (Aka, this is a polluted type signature
        // in exchange for a ~10x speedup because of the avoided clones. Previously, we had to
        // clone every time we recursed into an `Appl`.)
        T: Into<Self> + Clone,
    {
        match self {
            // [s/x] x := s
            // Only clone we have to do in this whole process is here.
            Self::Var(ref s) if s == replace => *self = with.clone().into(),

            // [s/x] y := y
            Self::Var(_) => (),

            // [s/x] (fn x => t) := (fn x => t)
            Self::Lam { ref param, .. } if param == replace => (),

            // [s/x] (fn y => t) := (fn z => [s/x] ([z/y] t)) for fresh z
            Self::Lam { param, rule } => {
                let new_var = get_fresh_ident(param);
                rule.subst(param, &new_var);
                rule.subst(replace, with);
                *param = new_var; // we need new_var for the param and the recursive subst
            }

            // [s/x] (t1 t2) := ([s/x] t1) ([s/x] t2)
            Self::Appl { left, right } => {
                left.subst(replace, with);
                right.subst(replace, with);
            }
        }
    }

    /// Check term equivalence under alpha-renaming.
    #[must_use]
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
                ctx.pop();
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
            assert_eq!(Var("x".into()).reduce(false), "x".into());
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

            assert_eq!(input.reduce(false), "z".into());
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

            assert_eq!(input.reduce(false), "y".into());
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
            assert_eq!(input.reduce(false), "a".into());
        }

        // takes a name, a string representing the term to be reduced, and a string representing
        // the expected normal form
        macro_rules! beta_reduction_tests { ($($name:ident: $input:expr, $expected:expr)*) => {
            $(
            #[test]
            fn $name() -> ParserResult<()> {
                // This is not a proper unit test because of the dependency on `to_term`, but it
                // makes tests _much_ easier to develop.
                assert!(to_term($input)?.reduce(false).alpha_equiv(&to_term($expected)?));
                Ok(())
            }
            )*

            mod bench {
                use super::to_term;

                extern crate test;
                use test::Bencher;
                $(
                #[bench]
                fn $name(b: &mut Bencher) {
                    b.iter(|| to_term($input).unwrap().reduce(false));
                }
                )*
            }
        }}

        beta_reduction_tests! {
            nested_sub: "(fn f => fn a => f) x", "fn a => x"
            order_matters: "(fn f => fn a => f (f a)) (fn q => r) a b", "r b"
            many_renames: "(fn f => fn y => fn x => x (y f)) y x f", "f (x y)"
            lazy_eval: "(fn t => fn e => t) x ((fn x => x x)(fn x => x x))", "x"
            y_combinator: "(fn g => ((fn y => g (y y)) (fn y => g (y y))))
                (fn f => fn x => x q (f (fn t => fn e => t))) (fn t => fn e => e)", "q"
            fibbit: "(fn n => (fn p => p (fn t => fn e => t)) (n (fn p => (fn a => fn b => fn s => s a b) ((fn p => p (fn t => fn e => e)) p) ((fn m => fn n => m (fn n => fn f => fn x => f (n f x)) n) ((fn p => p (fn t => fn e => t)) p) ((fn p => p (fn t => fn e => e)) p))) ((fn a => fn b => fn s => s a b) (fn f => fn x => x) ((fn n => fn f => fn x => f (n f x)) (fn f => fn x => x))))) (fn f => fn x => f (f (f (f (f (f (f (f (f (f x))))))))))", "fn f => fn x => f (f (f (f (f (f (f (f (f (f (f (f (f (f (f (f (f (f (f (f (f (f (f (f (f (f (f (f (f (f (f (f (f (f (f (f (f (f (f (f (f (f (f (f (f (f (f (f (f (f (f (f (f (f (f x))))))))))))))))))))))))))))))))))))))))))))))))))))))"
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
            let mut term = Lam {
                param: "x".into(),
                rule: "y".into(),
            };

            term.subst("y", &init);
            let expected = Lam {
                param: "z".into(),
                rule: Lam {
                    param: "y".into(),
                    rule: "x".into(), // this name is free in `init`, so should be preserved
                }
                .into(),
            };

            assert!(term.alpha_equiv(&expected));
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

            let mut out = term.clone();
            out.subst("z", &init); // z not in FV(term), so no sub necessary
            assert!(term.alpha_equiv(&out));
        }
    }
}

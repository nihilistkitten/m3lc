//! Booleans.
use lazy_static::lazy_static;

use crate::grammar::Term;
use Term::{Appl, Lam};

lazy_static! {
    static ref TRUE: Term = Lam {
        param: "t".into(),
        rule: Lam {
            param: "e".into(),
            rule: "t".into()
        }
        .into()
    };
    static ref FALSE: Term = Lam {
        param: "t".into(),
        rule: Lam {
            param: "e".into(),
            rule: "e".into()
        }
        .into()
    };
    static ref AND: Term = Lam {
        param: "a".into(),
        rule: Lam {
            param: "b".into(),
            rule: Appl {
                left: Appl {
                    left: "a".into(),
                    right: "b".into()
                }
                .into(),
                right: FALSE.clone().into()
            }
            .into()
        }
        .into()
    };
}

impl From<bool> for Term {
    fn from(b: bool) -> Self {
        if b {
            TRUE.clone()
        } else {
            FALSE.clone()
        }
    }
}

/// The `Term` is not a Church numeral.
#[derive(Debug)]
pub struct NotBool;

impl TryFrom<&Term> for bool {
    type Error = NotBool;

    fn try_from(term: &Term) -> Result<Self, Self::Error> {
        if term.alpha_equiv(&*TRUE) {
            Ok(true)
        } else if term.alpha_equiv(&*FALSE) {
            Ok(false)
        } else {
            Err(NotBool)
        }
    }
}

impl Term {
    pub fn and(self, other: Self) -> Self {
        Appl {
            left: Appl {
                left: AND.clone().into(),
                right: self.into(),
            }
            .into(),
            right: other.into(),
        }
        .reduce()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn true_and_true() {
        assert!(TRUE.clone().and(TRUE.clone()).alpha_equiv(&*TRUE));
    }

    #[test]
    fn true_and_false() {
        assert!(TRUE.clone().and(FALSE.clone()).alpha_equiv(&*FALSE));
    }

    #[test]
    fn false_and_true() {
        assert!(FALSE.clone().and(TRUE.clone()).alpha_equiv(&*FALSE));
    }

    #[test]
    fn false_and_false() {
        assert!(FALSE.clone().and(FALSE.clone()).alpha_equiv(&*FALSE));
    }
}

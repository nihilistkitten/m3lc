//! Parse a .m3lc file.
use crate::grammar::{ Term};
use Term::{Appl, Lam};

use pest::prec_climber as pcl;
use pest_consume::{match_nodes, Error, Parser};

#[derive(Parser)]
#[grammar = "m3lc.pest"]
pub struct M3LCParser;

/// A Result alias for Pest parsing errors.
pub type ParserResult<T> = std::result::Result<T, Error<Rule>>;

type Node<'a> = pest_consume::Node<'a, Rule, ()>;

lazy_static::lazy_static! {
    /// A precedence climber. Pest_consume's macros handle the actual work.
    static ref CLIMBER: pcl::PrecClimber<Rule> = pcl::PrecClimber::new(
        vec![
            pcl::Operator::new(Rule::juxa, pcl::Assoc::Left),
        ]
    );
}

#[pest_consume::parser]
impl M3LCParser {
    /// Parse an EOI.
    fn EOI(input: Node) -> ParserResult<()> {
        Ok(())
    }

    /// Parse an ident to a `String`.
    fn ident(input: Node) -> ParserResult<String> {
        Ok(input.as_str().into())
    }

    /// Parse a name to a `Term::Var`.
    ///
    /// name := ident
    fn var(input: Node) -> ParserResult<Term> {
        Ok(match_nodes!(input.into_children();
            [ident(x)] => x.into()
        ))
    }

    /// Parse a lam to a `Term::Lam`.
    ///
    /// lam = { "fn" ~ ident ~ "=>" ~ appl }
    fn lam(input: Node) -> ParserResult<Term> {
        Ok(match_nodes!(input.into_children();
            [ident(param), appl(rule)] => Lam{ param, rule: rule.into() },
        ))
    }

    /// Parse an appl to a `Term::Appl`.
    ///
    /// appl = { term ~ (juxa ~ term)* }
    ///
    /// Appls are parsed by CLIMBER as a left-heavy binary tree.
    #[prec_climb(term, CLIMBER)]
    #[allow(
        unused_variables,
        dead_code,
        clippy::needless_pass_by_value,
        clippy::unnecessary_wraps
    )] // these lints get confused by the macro
    fn appl(left: Term, op: Node, right: Term) -> ParserResult<Term> {
        Ok(Appl {
            left: left.into(),
            right: right.into(),
        })
    }

    /// Parse a term to a `Term`.
    ///
    /// term = { lam | var | "(" ~ appl ~ ")" }
    fn term(input: Node) -> ParserResult<Term> {
        Ok(match_nodes!(input.into_children();
            [appl(a)] => a,
            [lam(l)] => l,
            [var(x)] => x
        ))
    }
}

/// Parse a str to a term.
pub fn to_term(input: &str) -> ParserResult<Term> {
    let t = M3LCParser::parse(Rule::appl, input)?.single()?;
    M3LCParser::appl(t)
}

#[cfg(test)]
mod tests {
    use super::*;
    use Term::{Appl, Lam};

    /// macro to generate test cases for the parser
    /// takes a name, a string, and an AST
    macro_rules! parser_tests { ($($name:ident: $input:expr, $expected:expr)*) => {

        // modularized for name deduplication, concat_idents isn't on stable
        mod parse_term {
            use super::*;
            $(
            mod $name {
                use super::*;
                #[test]
                fn test() -> ParserResult<()> {
                    let term = to_term($input)?;
                    assert_eq!(
                        term,
                        $expected
                    );
                    Ok(())
                }
            }
            )*
        }
    }}

    parser_tests! {
        identity: "fn x => x", Lam{ param: "x".into(), rule: "x".into() }
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
        add: "fn n => fn m => n succ m", Lam{
            param: "n".into(),
            rule: Lam{
                param: "m".into(),
                rule: Appl{
                    left: Appl{
                        left: "n".into(),
                        right: "succ".into()
                    }.into(),
                    right: "m".into()
                }.into()
            }.into()
        }
        right_associative: "x (y z)", Appl{
            left: "x".into(),
            right: Appl{
                left: "y".into(),
                right: "z".into()
            }.into()
        }
        left_associative: "(x y) z", Appl{
            left: Appl{
                left: "x".into(),
                right: "y".into()
            }.into(),
            right: "z".into()
        }
    }
}

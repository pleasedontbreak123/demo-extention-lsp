//! ## Expression Parsing
//!
//! ```plain
//! <factor1> ::=
//!     | ( <expr> )
//!     | <ident>
//!     | <ident> ( <expr> , ... )
//!     | <literal>
//!
//! <factor2> ::=
//!     | <factor1>
//!     | + <factor2>
//!     | - <factor2>
//!     | ~ <factor2>
//!
//! <factor3> ::=
//!     | <factor2>
//!     | <factor2> ** <factor3>
//!
//! <factor4> ::=
//!     | <factor3>
//!     | <factor4> * <factor3>
//!     | <factor4> / <factor3>
//!
//! <factor5> ::=
//!     | <factor4>
//!     | <factor5> + <factor4>
//!     | <factor5> - <factor4>
//!
//! <factor6> ::=
//!     | <factor5>
//!     | <factor6> & <factor5>
//!
//! <factor7> ::=
//!     | <factor6>
//!     | <factor7> ^ <factor6>
//!
//! <factor8> ::=
//!     | <factor7>
//!     | <factor8> | <factor7>
//!
//! <factor9> ::=
//!     | <factor8>
//!     | <factor8> >= <factor8>
//!     | <factor8> <= <factor8>
//!     | <factor8> > <factor8>
//!     | <factor8> < <factor8>
//!
//! <factor10> ::=
//!     | <factor9>
//!     | <factor9> == <factor9>
//!     | <factor9> != <factor9>
//!
//! <factor11> ::=
//!     | <factor10>
//!     | <factor10> ? <factor11> : <factor11>
//!
//! <expr> ::= <factor11>
//! ```

use serde::Serialize;

use crate::{
    parse::{ParseError, ParseResult, TokenStream, TryParse},
    value::{NumberParser, SuffixNumberParser},
};

use super::{Atom, Name};

/// 数字字面值 `<literal>`
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Literal {
    pub value: f64,
}

impl<T> TryParse<Literal> for T
where
    T: TokenStream,
{
    fn try_parse(&mut self) -> ParseResult<Literal> {
        let mut neg = false;
        if self.matches_consume("-") {
            neg = true;
        }
        let token = self.token()?;
        let parser = SuffixNumberParser {};
        match parser.to_number(&token.raw) {
            Ok(value) => {
                self.consume();
                Ok(Literal {
                    value: if neg { -value } else { value },
                })
            }
            Err(_) => Err(ParseError::unexpected(
                "<string>",
                &token.raw,
                Some(token.column),
            )),
        }
    }
}

/// 标识符 `<ident>`
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Ident(pub Name);

impl<T> TryParse<Ident> for T
where
    T: TokenStream,
{
    fn try_parse(&mut self) -> ParseResult<Ident> {
        Ok(Ident(self.try_parse()?))
    }
}

/// ```plain
/// <factor1> ::=
///     | ( <expr> )
///     | <ident>
///     | <ident> ( <expr> , ... )
///     | <literal>
/// ```
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "type", content = "value")]
pub enum Factor1 {
    Parenthesis(Box<Expr>),
    Ident(Ident),
    Call { name: Ident, args: Vec<Expr> },
    Literal(Literal),
}

impl<T> TryParse<Factor1> for T
where
    T: TokenStream,
{
    fn try_parse(&mut self) -> ParseResult<Factor1> {
        // ( <expr> )
        if self.matches_consume("(") {
            let expr = Box::new(self.try_parse()?);
            self.expect(")")?;
            return Ok(Factor1::Parenthesis(expr));
        }

        // <ident>
        if let Some(ident) = self.try_parse()? {
            if self.matches_consume("(") {
                // <ident> ( <expr>, * )
                let mut exprs = Vec::new();
                while !self.matches_consume(")") {
                    exprs.push(self.try_parse()?);
                }
                return Ok(Factor1::Call {
                    name: ident,
                    args: exprs,
                });
            }
            return Ok(Factor1::Ident(ident));
        }

        Ok(Factor1::Literal(self.try_parse()?))
    }
}

/// ```plain
/// <factor2> ::=
///     | <factor1>
///     | + <factor2>
///     | - <factor2>
///     | ~ <factor2>
/// ```
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "type", content = "value")]
pub enum Factor2 {
    Base(Factor1),
    Pos(Box<Factor2>),
    Neg(Box<Factor2>),
    Not(Box<Factor2>),
}

impl<T: TokenStream> TryParse<Factor2> for T {
    fn try_parse(&mut self) -> ParseResult<Factor2> {
        if self.matches_consume("+") {
            return Ok(Factor2::Pos(Box::new(self.try_parse()?)));
        } else if self.matches_consume("-") {
            return Ok(Factor2::Neg(Box::new(self.try_parse()?)));
        } else if self.matches_consume("~") {
            return Ok(Factor2::Not(Box::new(self.try_parse()?)));
        }
        Ok(Factor2::Base(self.try_parse()?))
    }
}

/// ```plain
/// <factor3> ::=
///     | <factor2>
///     | <factor2> ** <factor3>
/// ```
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "type", content = "value")]
pub enum Factor3 {
    Base(Factor2),
    Pow(Box<Factor2>, Box<Factor3>),
}

impl<T: TokenStream> TryParse<Factor3> for T {
    fn try_parse(&mut self) -> ParseResult<Factor3> {
        let left = self.try_parse()?;
        if self.matches_consume("**") {
            let right = self.try_parse()?;
            return Ok(Factor3::Pow(Box::new(left), Box::new(right)));
        }
        Ok(Factor3::Base(left))
    }
}

/// ```plain
/// <factor4> ::=
///     | <factor3>
///     | <factor4> * <factor3>
///     | <factor4> / <factor3>
/// ```
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "type", content = "value")]
pub enum Factor4 {
    Base(Factor3),
    Mul(Box<Factor4>, Box<Factor3>),
    Div(Box<Factor4>, Box<Factor3>),
}

impl<T: TokenStream> TryParse<Factor4> for T {
    fn try_parse(&mut self) -> ParseResult<Factor4> {
        let mut left = Factor4::Base(self.try_parse()?);
        loop {
            if self.matches_consume("*") {
                let right = self.try_parse()?;
                left = Factor4::Mul(Box::new(left), Box::new(right));
            } else if self.matches_consume("/") {
                let right = self.try_parse()?;
                left = Factor4::Div(Box::new(left), Box::new(right));
            } else {
                break;
            }
        }
        Ok(left)
    }
}

/// ```plain
/// <factor5> ::=
///     | <factor4>
///     | <factor5> + <factor4>
///     | <factor5> - <factor4>
/// ```
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "type", content = "value")]
pub enum Factor5 {
    Base(Factor4),
    Add(Box<Factor5>, Box<Factor4>),
    Sub(Box<Factor5>, Box<Factor4>),
}

impl<T: TokenStream> TryParse<Factor5> for T {
    fn try_parse(&mut self) -> ParseResult<Factor5> {
        let mut left = Factor5::Base(self.try_parse()?);
        loop {
            if self.matches_consume("+") {
                let right = self.try_parse()?;
                left = Factor5::Add(Box::new(left), Box::new(right));
            } else if self.matches_consume("-") {
                let right = self.try_parse()?;
                left = Factor5::Sub(Box::new(left), Box::new(right));
            } else {
                break;
            }
        }
        Ok(left)
    }
}

/// ```plain
/// <factor6> ::=
///     | <factor5>
///     | <factor6> & <factor5>
/// ```
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "type", content = "value")]
pub enum Factor6 {
    Base(Factor5),
    And(Box<Factor6>, Box<Factor5>),
}

impl<T: TokenStream> TryParse<Factor6> for T {
    fn try_parse(&mut self) -> ParseResult<Factor6> {
        let mut node = Factor6::Base(self.try_parse()?);
        while self.matches_consume("&") {
            let right = self.try_parse()?;
            node = Factor6::And(Box::new(node), Box::new(right));
        }
        Ok(node)
    }
}

/// ```plain
/// <factor7> ::=
///     | <factor6>
///     | <factor7> ^ <factor6>
/// ```
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "type", content = "value")]
pub enum Factor7 {
    Base(Factor6),
    Xor(Box<Factor7>, Box<Factor6>),
}

impl<T: TokenStream> TryParse<Factor7> for T {
    fn try_parse(&mut self) -> ParseResult<Factor7> {
        let mut node = Factor7::Base(self.try_parse()?);
        while self.matches_consume("^") {
            let right = self.try_parse()?;
            node = Factor7::Xor(Box::new(node), Box::new(right));
        }
        Ok(node)
    }
}

/// ```plain
/// <factor8> ::=
///     | <factor7>
///     | <factor8> | <factor7>
/// ```
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "type", content = "value")]
pub enum Factor8 {
    Or(Box<Factor8>, Box<Factor7>),
    Base(Factor7),
}

impl<T: TokenStream> TryParse<Factor8> for T {
    fn try_parse(&mut self) -> ParseResult<Factor8> {
        let mut node = Factor8::Base(self.try_parse()?);
        while self.matches_consume("|") {
            let right = self.try_parse()?;
            node = Factor8::Or(Box::new(node), Box::new(right));
        }
        Ok(node)
    }
}

/// ```plain
/// <factor9> ::=
///     | <factor8>
///     | <factor8> >= <factor8>
///     | <factor8> <= <factor8>
///     | <factor8> > <factor8>
///     | <factor8> < <factor8>
/// ```
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "type", content = "value")]
pub enum Factor9 {
    Base(Factor8),
    Ge(Box<Factor8>, Box<Factor8>),
    Le(Box<Factor8>, Box<Factor8>),
    Gt(Box<Factor8>, Box<Factor8>),
    Lt(Box<Factor8>, Box<Factor8>),
}

impl<T: TokenStream> TryParse<Factor9> for T {
    fn try_parse(&mut self) -> ParseResult<Factor9> {
        let left = self.try_parse()?;
        if self.matches_consume(">=") {
            return Ok(Factor9::Ge(Box::new(left), Box::new(self.try_parse()?)));
        } else if self.matches_consume("<=") {
            return Ok(Factor9::Le(Box::new(left), Box::new(self.try_parse()?)));
        } else if self.matches_consume(">") {
            return Ok(Factor9::Gt(Box::new(left), Box::new(self.try_parse()?)));
        } else if self.matches_consume("<") {
            return Ok(Factor9::Lt(Box::new(left), Box::new(self.try_parse()?)));
        }
        Ok(Factor9::Base(left))
    }
}

/// ```plain
/// <factor10> ::=
///     | <factor9>
///     | <factor9> == <factor9>
///     | <factor9> != <factor9>
/// ```
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "type", content = "value")]
pub enum Factor10 {
    Base(Factor9),
    Eq(Box<Factor9>, Box<Factor9>),
    Neq(Box<Factor9>, Box<Factor9>),
}

impl<T: TokenStream> TryParse<Factor10> for T {
    fn try_parse(&mut self) -> ParseResult<Factor10> {
        let left = self.try_parse()?;
        if self.matches_consume("==") {
            return Ok(Factor10::Eq(Box::new(left), Box::new(self.try_parse()?)));
        } else if self.matches_consume("!=") {
            return Ok(Factor10::Neq(Box::new(left), Box::new(self.try_parse()?)));
        }
        Ok(Factor10::Base(left))
    }
}

/// ```plain
/// <factor11> ::=
///     | <factor10>
///     | <factor10> ? <factor11> : <factor11>
/// ```
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "type", content = "value")]
pub enum Factor11 {
    Ternary {
        cond: Box<Factor10>,
        then_branch: Box<Factor11>,
        else_branch: Box<Factor11>,
    },
    Base(Factor10),
}

impl<T: TokenStream> TryParse<Factor11> for T {
    fn try_parse(&mut self) -> ParseResult<Factor11> {
        let cond = self.try_parse()?;
        if self.matches_consume("?") {
            let then_branch = self.try_parse()?;
            self.expect(":")?;
            let else_branch = self.try_parse()?;
            return Ok(Factor11::Ternary {
                cond: Box::new(cond),
                then_branch: Box::new(then_branch),
                else_branch: Box::new(else_branch),
            });
        }
        Ok(Factor11::Base(cond))
    }
}

/// ```plain
/// <expr> ::= <factor11>
/// ```
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Expr(pub Factor11);

impl<T> TryParse<Expr> for T
where
    T: TokenStream,
{
    fn try_parse(&mut self) -> ParseResult<Expr> {
        Ok(Expr(self.try_parse()?))
    }
}

/// 一般数字
///
/// - `1.0`
/// - `{ <expression> }`
#[derive(Debug, Clone, PartialEq)]
pub enum Number {
    Literal(Literal),
    Expr(Expr),
}

impl<T> TryParse<Number> for T
where
    T: TokenStream,
{
    fn try_parse(&mut self) -> ParseResult<Number> {
        if self.matches_consume("{") {
            let expr = self.try_parse()?;
            self.expect("}")?;
            return Ok(Number::Expr(expr));
        }

        // FIXME: 后缀和单位都会被删掉！
        let literal = self.try_parse()?;
        Ok(Number::Literal(literal))
    }
}

impl Serialize for Number {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Number::Literal(literal) => serializer.serialize_f64(literal.value),
            Number::Expr(expr) => expr.serialize(serializer),
        }
    }
}

impl From<f64> for Literal {
    fn from(value: f64) -> Self {
        assert!(value.is_finite());
        Literal { value }
    }
}

impl From<i64> for Literal {
    fn from(value: i64) -> Self {
        Literal {
            value: value as f64,
        }
    }
}

impl From<Atom> for Ident {
    fn from(name: Atom) -> Self {
        Ident(Name(name))
    }
}

impl From<&str> for Ident {
    fn from(value: &str) -> Self {
        Ident(Name(Atom::from(value)))
    }
}

impl From<f64> for Number {
    fn from(value: f64) -> Self {
        Number::Literal(value.into())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Text(pub Atom);

impl<T> TryParse<Text> for T
where
    T: TokenStream,
{
    fn try_parse(&mut self) -> ParseResult<Text> {
        let token = self.token()?;
        if token.is_string() {
            self.consume();
            Ok(Text(token))
        } else {
            Err(ParseError {
                reason: format!("Expect a string, found `{}'", token.raw),
                position: Some(token.column),
            })
        }
    }
}

impl Text {
    pub fn to_string(&self) -> String {
        self.0.raw[1..self.0.raw.len() - 1].to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{spice_test_err, spice_test_ok};

    #[test]
    fn test_parse_number() {
        spice_test_ok!("114514", Number::from(114514.0));
        spice_test_ok!("12e40", Number::from(12e40));
        spice_test_ok!(
            "{alice}",
            Number::Expr(Expr(Factor11::Base(Factor10::Base(Factor9::Base(
                Factor8::Base(Factor7::Base(Factor6::Base(Factor5::Base(Factor4::Base(
                    Factor3::Base(Factor2::Base(Factor1::Ident(Ident::from("alice"))))
                )))))
            )))))
        );
        spice_test_ok!(
            "{bob}",
            Number::Expr(Expr(Factor11::Base(Factor10::Base(Factor9::Base(
                Factor8::Base(Factor7::Base(Factor6::Base(Factor5::Base(Factor4::Base(
                    Factor3::Base(Factor2::Base(Factor1::Ident(Ident::from("bob"))))
                )))))
            )))))
        );
        spice_test_err!("{alice", Number);
        spice_test_err!("n95", Number);
    }

    #[test]
    fn test_expression() {
        spice_test_ok!(
            "2*3-4/5",
            Expr(Factor11::Base(Factor10::Base(Factor9::Base(
                Factor8::Base(Factor7::Base(Factor6::Base(Factor5::Sub(
                    Box::new(Factor5::Base(Factor4::Mul(
                        Box::new(Factor4::Base(Factor3::Base(Factor2::Base(
                            Factor1::Literal(Literal::from(2))
                        )))),
                        Box::new(Factor3::Base(Factor2::Base(Factor1::Literal(
                            Literal::from(3)
                        ))))
                    ))),
                    Box::new(Factor4::Div(
                        Box::new(Factor4::Base(Factor3::Base(Factor2::Base(
                            Factor1::Literal(Literal::from(4))
                        )))),
                        Box::new(Factor3::Base(Factor2::Base(Factor1::Literal(
                            Literal::from(5)
                        ))))
                    ))
                ))))
            ))))
        );
    }
}

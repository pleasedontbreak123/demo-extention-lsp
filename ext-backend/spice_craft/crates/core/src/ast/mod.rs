//! 没写完，感觉没法抢救了

use std::fmt::Debug;

use command::Command;
use component::{Component};
use expression::Number;
use serde::Serialize;
use spice_proc_macro::TryParse;
use variable::OutputVariable;

use crate::parse::{ParseError, ParseResult, TokenStream, TryParse};

pub mod command;
pub mod component;
pub mod expression;
pub mod gain;
pub mod sweep;
pub mod transient;
pub mod variable;

/// 源代码中的片段
///
/// 注意：相等只比较字符串内容
#[derive(Clone)]
pub struct Atom {
    pub raw: String,
    pub column: (usize, usize),
    pub line: usize,
}

impl Serialize for Atom {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.raw)
    }
}

impl Debug for Atom {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}:{}:{}", self.raw, self.column.0, self.column.1)
    }
}

impl PartialEq for Atom {
    fn eq(&self, other: &Self) -> bool {
        self.raw == other.raw
    }
}

impl Eq for Atom {}

impl From<&str> for Atom {
    fn from(value: &str) -> Self {
        Self {
            raw: value.to_string(),
            column: Default::default(),
            line: Default::default(),
        }
    }
}

impl Atom {
    /// 转换成字符串
    pub fn to_string(&self) -> String {
        self.raw.clone()
    }

    /// 创建新的 Atom
    pub fn new(raw: &str, column: (usize, usize), line: usize) -> Self {
        Self {
            raw: raw.to_string(),
            column,
            line,
        }
    }

    /// 转换成大写
    pub fn to_uppercase(&self) -> String {
        self.raw.to_ascii_uppercase()
    }

    /// 是否是字符串
    pub fn is_string(&self) -> bool {
        self.raw.starts_with('"') && self.raw.ends_with('"')
    }
}

/// 源程序 AST
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Program {
    pub name: Option<Name>,
    pub instructions: Vec<Instruction>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "type", content = "data")]
pub enum Instruction {
    Command(Command),
    Component(Component),
}

//#[derive(Debug, Clone, PartialEq, Serialize)]
//#[serde(tag = "type", content = "data")]
//pub enum InstructionPart {
//    Command(CommandPartial),
//    Component(ComponentPartial),
//}

#[derive(Debug, Clone, PartialEq, Serialize, TryParse)]
#[grammar(variable, limits)]
pub struct PrintVariable {
    pub variable: OutputVariable,
    pub limits: Option<Limits>,
}

fn is_valid_digits_node_name(name: &str) -> bool {
    name.chars().all(|x| x.is_ascii_digit())
}

fn is_valid_node_name(name: &str) -> bool {
    name.chars()
        .all(|x| x.is_ascii_alphanumeric() || x == '_' || x == '$')
}

/// 节点名称
///
/// - `<node> ::= [0-9a-zA-Z]+`
/// - `<node> | [ <node> ]`
#[derive(Debug, Clone, PartialEq)]
pub struct Node(pub Atom);

// TODO: 核对 Node 语法
impl<T> TryParse<Node> for T
where
    T: TokenStream,
{
    fn try_parse(&mut self) -> ParseResult<Node> {
        let bracket = self.matches_consume("[");
        let token = self.token()?;
        if is_valid_node_name(&token.raw) {
            self.consume();
            if bracket {
                self.expect("]")?;
            }
            Ok(Node(token))
        } else {
            Err(ParseError {
                reason: format!("Expect `{}' to be a node", token.raw),
                position: Some(token.column),
            })
        }
    }
}

impl Serialize for Node {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.0.to_string())
    }
}

/// 用来匹配明确声明的节点
///
/// 节点可以是 \w+ 的任意组合，所有有可能和 Name 混淆，所以在一些特殊地方会在两边加方括号的形式来明确声明这是 Node 而不是 Name。
///
/// - `<digits node>`
/// - `[ <node> ]`
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ExplicitNode(pub Node);

impl<T> TryParse<ExplicitNode> for T
where
    T: TokenStream,
{
    fn try_parse(&mut self) -> ParseResult<ExplicitNode> {
        if self.matches_consume("[") {
            let token = self.token()?;
            if is_valid_node_name(&token.raw) {
                self.consume();
                self.expect("]")?;
                Ok(ExplicitNode(Node(token)))
            } else {
                Err(ParseError {
                    reason: format!("Expect `{}' to be a node", token.raw),
                    position: Some(token.column),
                })
            }
        } else {
            let token = self.token()?;
            if is_valid_digits_node_name(&token.raw) {
                self.consume();
                Ok(ExplicitNode(Node(token)))
            } else {
                Err(ParseError {
                    reason: format!(
                        "Expect `{}' to be an explicit node (only contains digits, or contained in `[]')",
                        token.raw
                    ),
                    position: Some(token.column),
                })
            }
        }
    }
}

/// 元件名称 / 模型名称
///
/// - `[a-zA-Z][a-zA-Z0-9]*`
#[derive(Debug, Clone, PartialEq)]
pub struct Name(pub Atom);

impl<T> TryParse<Name> for T
where
    T: TokenStream,
{
    fn try_parse(&mut self) -> ParseResult<Name> {
        // [a-zA-Z][a-zA-Z0-9_]*
        let token = self.token()?;
        let mut chars = token.raw.chars();
        if let Some(start) = chars.next() {
            if start.is_ascii_alphabetic() && chars.all(|x| x.is_ascii_alphanumeric() || x == '_') {
                self.consume();
                return Ok(Name(token));
            }
        }
        Err(ParseError {
            reason: format!("Expect `{}' to be a name", token.raw),
            position: Some(token.column),
        })
    }
}

impl Serialize for Name {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.0.to_string())
    }
}

/// AC 扫描类型
#[derive(Debug, Clone, PartialEq, TryParse, Serialize)]
#[serde(tag = "type")]
pub enum AcType {
    /// 线性扫描
    #[matches("LIN")]
    Lin,
    /// 倍频扫描（6 倍）
    #[matches("OCT")]
    Oct,
    /// 十倍扫描
    #[matches("DEC")]
    Dec,
}

/// `<name>=<value>`
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Param<T: Serialize> {
    pub name: Name,
    pub value: T,
}

impl<T, R> TryParse<Param<R>> for T
where
    R: Serialize,
    T: TokenStream + TryParse<Name> + TryParse<R>,
{
    fn try_parse(&mut self) -> ParseResult<Param<R>> {
        let name = self.try_parse()?;
        self.expect("=")?;
        let value = self.try_parse()?;
        Ok(Param { name, value })
    }
}

/// `<name>=<value>`
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Pairs<T: Serialize, R: Serialize> {
    pub name: T,
    pub value: R,
}

impl<T, R, G> TryParse<Pairs<R, G>> for T
where
    R: Serialize,
    G: Serialize,
    T: TokenStream + TryParse<R> + TryParse<G>,
{
    fn try_parse(&mut self) -> ParseResult<Pairs<R, G>> {
        let name = self.try_parse()?;
        self.expect("=")?;
        let value = self.try_parse()?;
        Ok(Pairs { name, value })
    }
}

/// - `<name>`
/// - `<name>=<value>`
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Options<T: Serialize> {
    pub name: Name,
    pub value: Option<T>,
}

impl<T, R> TryParse<Options<R>> for T
where
    R: Serialize,
    T: TokenStream + TryParse<Name> + TryParse<R>,
{
    fn try_parse(&mut self) -> ParseResult<Options<R>> {
        let name = self.try_parse()?;
        if self.matches_consume("=") {
            let value = Some(self.try_parse()?);
            Ok(Options { name, value })
        } else {
            Ok(Options { name, value: None })
        }
    }
}

/// `(<lower>, <upper>)`
#[derive(Debug, Clone, PartialEq, Serialize, TryParse)]
#[grammar("(", lower, upper, ")")]
pub struct Limits {
    pub lower: Number,
    pub upper: Number,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct OptionParenthesis<T: Serialize>(pub T);

impl<T, R> TryParse<OptionParenthesis<R>> for T
where
    R: Serialize,
    T: TokenStream + TryParse<R>,
{
    fn try_parse(&mut self) -> ParseResult<OptionParenthesis<R>> {
        if self.matches_consume("(") {
            let inner: R = self.try_parse()?;
            self.expect(")")?;
            Ok(OptionParenthesis(inner))
        } else {
            Ok(OptionParenthesis(self.try_parse()?))
        }
    }
}

#[cfg(test)]
mod tests {

    use expression::{Expr, Ident};

    use crate::{
        ast::expression::{
            Factor1, Factor10, Factor11, Factor2, Factor3, Factor4, Factor5, Factor6, Factor7,
            Factor8, Factor9,
        },
        spice_test_err, spice_test_ok,
    };

    use super::*;

    #[test]
    fn test_parse_node() {
        spice_test_ok!("1", Node(Atom::from("1")));
        spice_test_ok!("a", Node(Atom::from("a")));
        spice_test_ok!("[1]", Node(Atom::from("1")));
        spice_test_ok!("[a]", Node(Atom::from("a")));
    }

    #[test]
    fn test_parse_node_weird() {
        spice_test_ok!("$G", Node(Atom::from("$G")));
        spice_test_err!("@b", Node);
    }

    #[test]
    fn test_parse_explicit_node() {
        spice_test_ok!("1", ExplicitNode(Node(Atom::from("1"))));
        spice_test_err!("a", ExplicitNode);
        spice_test_ok!("[1]", ExplicitNode(Node(Atom::from("1"))));
        spice_test_ok!("[a]", ExplicitNode(Node(Atom::from("a"))));
    }

    #[test]
    fn test_parse_param_value() {
        spice_test_ok!(
            "alice=1919810E0",
            Param {
                name: Name(Atom::from("alice")),
                value: Number::from(1919810.0),
            }
        );
        spice_test_err!("alice=", Param<Number>);
        spice_test_err!("alice 1", Param<Number>);
        spice_test_err!("=1", Param<Number>);
        spice_test_err!("1=1", Param<Number>);
    }

    #[test]
    fn test_parse_param_number_expression() {
        spice_test_ok!(
            "alice={bob}",
            Param {
                name: Name(Atom::from("alice")),
                value: Number::Expr(Expr(Factor11::Base(Factor10::Base(Factor9::Base(
                    Factor8::Base(Factor7::Base(Factor6::Base(Factor5::Base(Factor4::Base(
                        Factor3::Base(Factor2::Base(Factor1::Ident(Ident::from("bob"))))
                    )))))
                ))))),
            }
        );
    }
}

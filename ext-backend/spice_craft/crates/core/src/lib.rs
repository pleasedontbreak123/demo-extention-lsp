//! Spice 解析器核心库
//!
//! 分为 ast 和 lexer 两个子模块。

use std::{collections::HashSet, path::PathBuf};

use ast::Program;
use lexer::SpiceLexer;
use parse::{ParseResult, SpiceFileParser, TryParse};

/// 抽象语法树
pub mod ast;
/// TODO: 作为对外的结构体
pub mod grammar;
/// 词法解析器
pub mod lexer;
/// 语法解析工具
pub mod parse;
/// 值解析
pub mod value;

/// 解析字符串，变成 AST
pub fn try_parse_program(content: &str) -> ParseResult<Program> {
    let tokens = SpiceLexer::tokenize(content);
    let mut parser = SpiceFileParser::new(&tokens);
    parser.parse()
}

// // 'a in function body outlives function.
// pub fn parse_line<'b, T: PartialEq>(l: &'b str) -> Result<T, parse::ParseError>
// where
//     for<'c> SpiceLineParser<'c>: TryParse<T>,
// {
//     let tokens = SpiceLexer::tokenize(l);
//     let mut parser = SpiceLineParser::new(&tokens[0]);
//     parser.try_parse()
// }

// pub fn check_line<'b, T: PartialEq>(l: &'b str, rhs: T) -> bool
// where
//     for<'c> SpiceLineParser<'c>: TryParse<T>,
// {
//     parse_line(l).map(|x| x == rhs).unwrap_or(false)
// }

// pub fn parse_file(content: &str) -> Result<Program, ParseError> {
//     let tokens = SpiceLexer::tokenize(content);
//     SpiceFileParser::new(&tokens).parse()
// }

#[cfg(test)]
#[macro_export]
macro_rules! spice_test_ok {
    ($tkns:expr, $mths:expr) => {{
        let tokens = $crate::lexer::SpiceLexer::tokenize($tkns);
        let mut parser = $crate::parse::SpiceLineParser::new(&tokens[0]);
        let result = parser.try_parse();
        assert_eq!(result, Ok($mths));
        assert!(parser.is_eof());
    }};
}

#[cfg(test)]
#[macro_export]
macro_rules! spice_test_err {
    ($tkns:expr, $type:ty) => {{
        let tokens = $crate::lexer::SpiceLexer::tokenize($tkns);
        let mut parser = $crate::parse::SpiceLineParser::new(&tokens[0]);
        let result: $crate::parse::ParseResult<$type> = parser.try_parse();
        assert!(result.is_err());
    }};
}

type PSpiceResult<T> = Result<T, Diagnostic<usize>>;
use codespan_reporting::{
    diagnostic::{Diagnostic, Label},
    files::SimpleFiles,
    term::{
        self,
        termcolor::{ColorChoice, StandardStream},
        Config,
    },
};

pub struct Worker {
    files: SimpleFiles<String, String>,
    visit: HashSet<PathBuf>,
}

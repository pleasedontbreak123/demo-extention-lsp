
//use spice_parser_core::{ast::{component, expression::Text, Atom, Name}, grammar::*, parse::ExposeNodes, *};
use tower_lsp::lsp_types::Range;
/// 符号的种类
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SpiceSymbolKind {
    Component,       // 组件类型，例如 R、C、L、V...
    Command,         // 分析命令，如 AC、DC、TRAN
    SubCircuit,      // 子电路（XComponent）引用的子电路名
    Model,           // 模型引用（例如 diode model）
    CircuitName,     // 主电路名（Program.name）
}

/// 单个符号的信息
#[derive(Debug, Clone)]
pub struct Symbol {
    pub name: String,
    pub range: Range,       // 精确的位置
    pub kind: SpiceSymbolKind, 
    pub container: Option<String>, // 所属容器（函数/模块名）  // 函数、变量、结构体等
}
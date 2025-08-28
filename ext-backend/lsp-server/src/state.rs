use spice_parser_core::ast::Program;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_lsp::lsp_types::Url;
use crate::symbol_info::table::SymbolTable;

#[derive(Default, Clone)]
pub struct ServerState {
    pub documents: HashMap<Url, DocumentState>,
    //pub global_symbols: SymbolIndex,
}

#[derive(Default, Clone)]
pub struct DocumentState {
    pub text: String,
    pub ast: Option<Program>,
    pub symbols: Option<SymbolTable>, // 语义信息
}



/// 共享引用类型，保证并发安全
pub type SharedServerState = Arc<Mutex<ServerState>>;

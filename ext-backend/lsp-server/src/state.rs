use spice_parser_core::ast::Program;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_lsp::lsp_types::Url;

#[derive(Default, Clone)]
pub struct ServerState {
    pub documents: HashMap<Url, String>,
    pub asts: HashMap<Url, Program>,
}

/// 共享引用类型，保证并发安全
pub type SharedServerState = Arc<Mutex<ServerState>>;

use crate::handler;
use crate::state::{ServerState, SharedServerState};
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, jsonrpc::Result};

pub struct Server {
    pub client: Client,
    pub state: SharedServerState,
}

impl Server {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            state: Arc::new(Mutex::new(ServerState::default())),
        }
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Server {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        // 告诉客户端我们支持的能力
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL, // 简化：使用全量同步
                )),
                completion_provider: Some(CompletionOptions {
                    resolve_provider: Some(false),
                    trigger_characters: Some(vec![".".to_string(), ":".to_string(), " ".to_string(), "R".to_string(), "C".to_string(), "L".to_string()]),
                    all_commit_characters: None,
                    work_done_progress_options: WorkDoneProgressOptions::default(),
                    completion_item: None,
                }),
                ..Default::default()
            },
            server_info: Some(ServerInfo {
                name: "my-lsp-server".into(),
                version: Some("0.1.0".into()),
            }),
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "LSP server initialized!")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        self.client.log_message(MessageType::INFO, "Server is shutting down").await;
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
         self.client
            .log_message(MessageType::INFO, &format!("did_open: {:?}", params.text_document.uri))
            .await;
        handler::diagnostics::on_did_open(&self.client, self.state.clone(), params).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
         self.client
            .log_message(MessageType::INFO, &format!("did_change: {:?}", params.text_document.uri))
            .await;
        handler::diagnostics::on_did_change(&self.client, self.state.clone(), params).await;
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
          self.client
            .log_message(MessageType::INFO, &format!("completion request at {:?}", params.text_document_position))
            .await;
        let response = handler::completion::on_completion(&self.client,self.state.clone(), params).await;
        //  self.client
        //     .log_message(MessageType::INFO, &format!("completion responce at {:?}", response))
        //     .await;
        response
    }
}

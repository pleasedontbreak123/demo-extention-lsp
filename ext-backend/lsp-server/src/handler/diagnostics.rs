use crate::state::{DocumentState, SharedServerState};
use spice_parser_core::try_parse_program;
use tower_lsp::Client;
use tower_lsp::lsp_types::*;

pub async fn on_did_open(
    client: &Client,
    state: SharedServerState,
    params: DidOpenTextDocumentParams,
) {
    let uri = params.text_document.uri.clone();
    let text = params.text_document.text;

    {
        let mut s = state.lock().await;
        //s.documents.insert(uri.clone(), text.clone());
        let doc_state = DocumentState {
            text,
            ast: None,
            symbols: None,
        };
        s.documents.insert(uri.clone(), doc_state);
    }

    reparse_and_publish(client, state, uri).await;
}

pub async fn on_did_change(
    client: &Client,
    state: SharedServerState,
    params: DidChangeTextDocumentParams,
) {
    let uri = params.text_document.uri.clone();

    // 这里使用 TextDocumentSyncKind::FULL，因此只取第一条变更的 text
    let new_text = params
        .content_changes
        .first()
        .map(|c| c.text.clone())
        .unwrap_or_default();

    {
        let mut s = state.lock().await;
        if let Some(doc) = s.documents.get_mut(&uri) {
            doc.text = new_text;
            doc.ast = None; // 等待重新解析
        }
    }

    reparse_and_publish(client, state, uri).await;
}

async fn reparse_and_publish(client: &Client, state: SharedServerState, uri: Url) {
    let source = {
        let s = state.lock().await;
        s.documents
            .get(&uri)
            .map(|doc| doc.text.clone())
            .unwrap_or_default()
    };

    match try_parse_program(&source) {
        Ok(program) => {
            // 缓存 AST
            {
                let mut s = state.lock().await;
                if let Some(doc) = s.documents.get_mut(&uri) {
                    doc.ast = Some(program); // 等待重新解析
                }
            }

            // 清空诊断（或基于 AST 生成真正的诊断）
            client.publish_diagnostics(uri, vec![], None).await;

            client
                .log_message(MessageType::INFO, "Parsed successfully")
                .await;
        }
        Err(err) => {
            let (line, col) = err.position.unwrap_or((0, 0));

            let line_text = source.lines().nth(line).unwrap_or("");
            let word_len = extract_word(line_text, col).unwrap_or(1); // 默认长度 1

            let diag = Diagnostic {
                range: Range {
                    start: Position::new(line as u32, col as u32),
                    end: Position::new(line as u32, col as u32 + word_len as u32),
                },
                severity: Some(DiagnosticSeverity::ERROR),
                message: err.reason,
                ..Default::default()
            };
            client
                .publish_diagnostics(uri.clone(), vec![diag], None)
                .await;

            client.log_message(MessageType::ERROR, "Parse failed").await;
        }
    }
}

fn extract_word(line_text: &str, location: usize) -> Option<usize> {
    let chars: Vec<char> = line_text.chars().collect();

    // 检查location是否在有效范围内
    if location >= chars.len() {
        return Some(1); // 默认长度1
    }

    if chars[location] != ' ' {
        let mut start = location;
        while start > 0 && chars[start - 1] != ' ' {
            start -= 1;
        }

        let mut end = location;
        while end < chars.len() && chars[end] != ' ' {
            end += 1;
        }

        //左闭右开
        Some(end - start)
    } else {
        None
    }
}

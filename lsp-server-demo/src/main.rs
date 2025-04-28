use lsp_server::{Connection, Message, Request};
use lsp_types::{
    CompletionItem, CompletionItemKind, InitializeParams, InitializeResult, 
    ServerCapabilities, TextDocumentSyncCapability, TextDocumentSyncKind,
    CompletionParams, CompletionResponse
};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    log::info!("LSP服务器启动中...");

    // 启动 stdio LSP 通信
    let (connection, io_threads) = Connection::stdio();
    log::info!("LSP服务器已启动(stdio)");

    // 等待 client 发来的 initialize 请求
    let (id, params) = connection.initialize_start()?;
    log::info!("收到初始化请求");
    let _params: InitializeParams = serde_json::from_value(params)?;

    let capabilities = ServerCapabilities {
        text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::FULL)),
        completion_provider: Some(lsp_types::CompletionOptions {
            resolve_provider: Some(true),
            ..Default::default()
        }),
        ..Default::default()
    };

    let result = InitializeResult {
        capabilities,
        server_info: None,
    };

    connection.initialize_finish(id, serde_json::to_value(result)?)?;
    log::info!("初始化完成");

    // 主事件循环
    for msg in &connection.receiver {
        match msg {
            Message::Request(req) => {
                if connection.handle_shutdown(&req)? {
                    log::info!("收到关闭请求，准备退出");
                    break;
                }

                log::info!("收到请求: {:?}", req);
                let resp = match req {
                    Request { id, method, params, .. } => {
                        match method.as_str() {
                            "textDocument/completion" => {
                                log::info!("处理补全请求");
                                let params: CompletionParams = serde_json::from_value(params)?;
                                let completions = handle_completion_request(&params);
                                log::info!("返回补全项: {:?}", completions);
                                lsp_server::Response::new_ok(id, completions)
                            },
                            "completionItem/resolve" => {
                                log::info!("处理补全项解析请求");
                                let item: CompletionItem = serde_json::from_value(params.clone())?;
                                log::info!("解析补全项: {:?}", item);
                                let resolved_item = resolve_completion_item(item);
                                log::info!("返回解析后的补全项: {:?}", resolved_item);
                                lsp_server::Response::new_ok(id, resolved_item)
                            },
                            _ => {
                                log::warn!("未处理的请求方法: {}", method);
                                let code = lsp_server::ErrorCode::MethodNotFound as i32;
                                let message = format!("Unhandled method: {method}");
                                lsp_server::Response::new_err(id, code, message)
                            }
                        }
                    }
                };
                connection.sender.send(Message::Response(resp))?;
            }
            Message::Notification(notif) => {
                log::info!("收到通知: {:?}", notif);
            }
            Message::Response(resp) => {
                log::info!("收到响应: {:?}", resp);
            }
        }
    }

    log::info!("LSP服务器正在关闭...");
    io_threads.join()?;
    log::info!("LSP服务器已关闭");
    Ok(())
}

// 处理补全请求
fn handle_completion_request(_params: &CompletionParams) -> CompletionResponse {
    // 在此返回一组补全项
    let completions = vec![
        CompletionItem {
            label: "fn".to_string(),
            kind: Some(CompletionItemKind::TEXT),
            ..Default::default()
        },
        CompletionItem {
            label: "蛤蛤蛤哈哈哈".to_string(),
            kind: Some(CompletionItemKind::TEXT),
            ..Default::default()
        },
        CompletionItem {
            label: "mut".to_string(),
            kind: Some(CompletionItemKind::TEXT),
            ..Default::default()
        },
        CompletionItem {
            label: "if".to_string(),
            kind: Some(CompletionItemKind::TEXT),
            ..Default::default()
        },
        CompletionItem {
            label: "oiiaioiiiaioiiiaii".to_string(),
            kind: Some(CompletionItemKind::TEXT),
            ..Default::default()
        },
        CompletionItem {
            label: "家家悦".to_string(),
            kind: Some(CompletionItemKind::TEXT),
            ..Default::default()
        },
        CompletionItem {
            label: "else".to_string(),
            kind: Some(CompletionItemKind::TEXT),
            ..Default::default()
        },
    ];

    CompletionResponse::Array(completions)
}

// 处理补全项解析
fn resolve_completion_item(mut item: CompletionItem) -> CompletionItem {
    // 根据标签添加不同的文档内容
    match item.label.as_str() {
        "fn" => {
            item.detail = Some("函数关键字".to_string());
            item.documentation = Some(lsp_types::Documentation::String(
                "用于声明函数，例如：fn foo() {}".to_string()
            ));
        },
        "蛤蛤蛤哈哈哈" => {
            item.detail = Some("关键字".to_string());
            item.documentation = Some(lsp_types::Documentation::String(
                "永远怀念".to_string()
            ));
        },
        "mut" => {
            item.detail = Some("可变修饰符".to_string());
            item.documentation = Some(lsp_types::Documentation::String(
                "用于声明可变变量，例如：let mut x = 5;".to_string()
            ));
        },
        "if" => {
            item.detail = Some("条件语句".to_string());
            item.documentation = Some(lsp_types::Documentation::String(
                "用于条件分支，例如：if x > 5 { ... }".to_string()
            ));
        },
        "else" => {
            item.detail = Some("else分支".to_string());
            item.documentation = Some(lsp_types::Documentation::String(
                "用于if语句的替代分支，例如：if x > 5 { ... } else { ... }".to_string()
            ));
        },
        "家家悦" => {
            item.detail = Some("坑".to_string());
            item.documentation = Some(lsp_types::Documentation::String(
                "keng".to_string()
            ));
        },
        "oiiaioiiiaioiiiaii" => {
            item.detail = Some("too young to simple +1s ".to_string());
            item.documentation = Some(lsp_types::Documentation::String(
                "一句顶一万".to_string()
            ));
        },
        _ => {}
    }
    
    item
}

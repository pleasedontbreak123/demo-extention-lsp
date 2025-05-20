mod handler;

use lsp_server::{Connection, Message, Request};
use lsp_types::{
    CompletionItem, CompletionItemKind, CompletionParams, CompletionResponse, DefinitionOptions, Hover, HoverContents, HoverParams, HoverProviderCapability, InitializeParams, InitializeResult, Location, MarkupContent, MarkupKind, OneOf, SemanticTokenModifier, SemanticTokenType, SemanticTokensFullOptions, SemanticTokensLegend, SemanticTokensOptions, SemanticTokensServerCapabilities, ServerCapabilities, TextDocumentPositionParams, TextDocumentSyncCapability, TextDocumentSyncKind
};
use std::error::Error;
use env_logger::Env;

fn main() -> Result<(), Box<dyn Error>> {
    // 配置日志，只显示 INFO 级别
    env_logger::Builder::from_env(Env::default().default_filter_or("info"))
        .format_timestamp(None)
        .format_level(true)
        .format_target(true)
        .init();
    
    log::info!("LSP服务器启动中...");

    // 启动 stdio LSP 通信
    let (connection, io_threads) = Connection::stdio();
    log::info!("LSP服务器已启动(stdio)");

    // 等待 client 发来的 initialize 请求
    let (id, params) = connection.initialize_start()?;
    //log::info!("收到初始化请求");
    log::info!("收到初始化请求: {}", serde_json::to_string_pretty(&params).unwrap());
    let _params: InitializeParams = serde_json::from_value(params)?;

    let capabilities = ServerCapabilities {
        text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::FULL)),//文档同步

        completion_provider: Some(lsp_types::CompletionOptions {
            resolve_provider: Some(true),
            ..Default::default()
        }),//代码补全
        // 悬浮提示功能
        hover_provider: Some(HoverProviderCapability::Simple(true)),


        // 跳转到定义功能
        definition_provider: Some(OneOf::Left(true)),
        // 语义高亮（语法高亮）功能
        semantic_tokens_provider: Some(SemanticTokensOptions {
            legend: SemanticTokensLegend {
             token_types: vec![
                SemanticTokenType::KEYWORD,
                SemanticTokenType::VARIABLE,
                SemanticTokenType::FUNCTION,
                SemanticTokenType::COMMENT,
                SemanticTokenType::STRING,
                // 你可以根据实际需求添加更多类型
            ],
            token_modifiers: vec![
                SemanticTokenModifier::DECLARATION,
                SemanticTokenModifier::STATIC,
            ],
            },
            full: Some(SemanticTokensFullOptions::Bool(true)), // 支持全量更新
            range: Some(true), // 支持范围更新
            work_done_progress_options: Default::default(),
        }.into()),

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
                                let response = lsp_server::Response::new_ok(id, completions);
                                // 完整输出响应内容
                                log::info!("发送响应: {}", serde_json::to_string_pretty(&response)?);
                                response
                            },
                            "completionItem/resolve" => {
                                let item: CompletionItem = serde_json::from_value(params.clone())?;
                                let resolved_item = resolve_completion_item(item);
    
                                let response = lsp_server::Response::new_ok(id, resolved_item);
    
                                log::info!("发送响应: {}", serde_json::to_string_pretty(&response)?);
                                response
                            },
                            "textDocument/hover" => {
                                let hover_params: HoverParams = serde_json::from_value(params.clone())?;

                                // 示例处理函数（你需要实现这个）
                                let result = handle_hover(hover_params);

                                let response = lsp_server::Response::new_ok(id, result);
                                log::info!("发送响应: {}", serde_json::to_string_pretty(&response)?);
                                response
                            }
                            _ => {
                                let code = lsp_server::ErrorCode::MethodNotFound as i32;
                                let message = format!("Unhandled method: {method}");
                                log::info!("未处理的方法: {}, code: {}, message: {}", method, code, message);
                                let response = lsp_server::Response::new_ok(id, CompletionResponse::Array(vec![]));
                                // 完整输出响应内容
                                log::info!("发送响应: {}", serde_json::to_string_pretty(&response)?);
                                response
                            }
                        }
                    }
                };
                connection.sender.send(Message::Response(resp))?;
            }
            Message::Notification(notif) => {
                //log::info!("收到通知: {:?}", notif);
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


fn extract_word(line_text: &str, location: usize) -> Option<String>{
    let chars: Vec<char> = line_text.chars().collect();

    if chars[location]!= ' ' {
        let mut start = location;
        while start > 0 && chars[start-1] != ' ' {
            start-=1;
        }

        let mut end = location;
        while end < chars.len() && chars[end] != ' ' {
            end+=1;
        }

        //左闭右开
        Some(chars[start..end].iter().collect())
    }else {
        None
    }
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
            item.documentation = Some(lsp_types::Documentation::MarkupContent(
                lsp_types::MarkupContent {
                    kind: lsp_types::MarkupKind::Markdown,
                    value: "用于 `if` 语句的替代分支，例如：\n```rust\nif x > 5 {\n    // do something\n} else {\n    // alternative\n}\n```".to_string(),
                },
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

//处理悬浮提示
/// 简单的 hover 实现：根据光标位置返回固定说明
pub fn handle_hover(params: HoverParams) -> Option<Hover> {
    let TextDocumentPositionParams { position, .. } = params.text_document_position_params;

    // 你可以根据 position.line 和 position.character 决定返回内容
    // 这里以示例第 7 行第 0 列为例，假设是悬停在 fn 上
    //position.line == 7 && position.character == 0
    if true {
        Some(Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: "这是 Rust 中定义函数的关键字：`fn`。".to_string(),
            }),
            range: None, // 可选，提供悬停词的范围
        })
    } else {
        None
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_word() {
        let line = "hello world rust";
        assert_eq!(extract_word(line, 1), Some("hello".to_string()));
        assert_eq!(extract_word(line, 7), Some("world".to_string()));
        assert_eq!(extract_word(line, 13), Some("rust".to_string()));
        assert_eq!(extract_word(line, 0), Some("hello".to_string()));
        assert_eq!(extract_word(line, 15), Some("rust".to_string()));
        assert_eq!(extract_word(line, 5), None);
        //println!("{:?}",extract_word(line, 15));

        let line2 = ".TRAN 1M 1 ";
        assert_eq!(extract_word(line2, 2), Some(".TRAN".to_string()));
    }

   
}
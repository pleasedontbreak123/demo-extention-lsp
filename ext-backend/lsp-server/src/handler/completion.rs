use crate::state::SharedServerState;
use spice_parser_core::ast::component::ComponentPartial;
use spice_parser_core::ast::Atom;
use spice_parser_core::lexer::SpiceLexer;
use spice_parser_core::parse::{PartialParse, SpiceLineParser};
use std::fs;
use tower_lsp::Client;
use tower_lsp::lsp_types::{Documentation, InsertTextFormat, *};

pub async fn on_completion(
    client: &Client,
    state: SharedServerState,
    params: CompletionParams,
) -> Result<Option<CompletionResponse>, tower_lsp::jsonrpc::Error> {
    // 1. 获取文档 URI
    let uri = params.text_document_position.text_document.uri;

    // 2. 获取文档内容
    let (maybe_source, existed) = {
        let s = state.lock().await;
        let opt = s.documents.get(&uri).cloned();
        (opt, s.documents.contains_key(&uri))
    };
    if !existed {
        client
            .log_message(
                MessageType::INFO,
                &format!("completion: document not found in cache: {:?}", uri),
            )
            .await;
    }
    let mut source = maybe_source.unwrap_or_default();
    if source.text.is_empty() {
        if let Ok(path) = uri.to_file_path() {
            if let Ok(content) = fs::read_to_string(&path) {
                client
                    .log_message(
                        MessageType::INFO,
                        &format!("completion: loaded from disk: {:?}", path),
                    )
                    .await;
                source.text = content;
                // 可选：写回缓存，避免后续重复读取
                {
                    let mut s = state.lock().await;
                    if let Some(doc) = s.documents.get_mut(&uri) {
                        doc.text = source.text.clone();
                    } else {
                        s.documents.insert(uri.clone(), source.clone());
                    }
                }
            }
        }
    }

    // 3. 获取光标所在行和列
    let line = params.text_document_position.position.line as usize;
    let col = params.text_document_position.position.character as usize;

    let total_lines = source.text.lines().count();
    if total_lines == 0 {
        client
            .log_message(
                MessageType::INFO,
                &format!("completion: empty document text, uri={:?}", uri),
            )
            .await;
        return Ok(Some(CompletionResponse::Array(vec![])));
    }
    let safe_line = if line >= total_lines {
        total_lines - 1
    } else {
        line
    };
    let line_text = source.text.lines().nth(safe_line).unwrap_or("");
    client
        .log_message(
            MessageType::INFO,
            &format!(
                "completion safe_line={} total_lines={} extracted='{}'",
                safe_line, total_lines, line_text
            ),
        )
        .await;

    client
        .log_message(
            MessageType::INFO,
            &format!("completion line: {:?}", line_text),
        )
        .await;

    let tokens = SpiceLexer::tokenize(line_text);
    if tokens.is_empty() {
        // 如果没有tokens，提供默认的补全选项
        let completions = vec!["R".to_string(), "C".to_string(), "L".to_string()];
        let items: Vec<CompletionItem> = completions
            .into_iter()
            .map(|label| CompletionItem {
                label: label.clone(),
                kind: Some(CompletionItemKind::TEXT),
                detail: Some("SPICE Component".to_string()),
                documentation: Some(Documentation::String("SPICE电路元件".to_string())),
                insert_text: Some(label),
                insert_text_format: Some(InsertTextFormat::PLAIN_TEXT),
                ..Default::default()
            })
            .collect();
        return Ok(Some(CompletionResponse::Array(items)));
    }
    let mut parser = SpiceLineParser::new(&tokens.first().unwrap());

    let completions = match <SpiceLineParser as PartialParse<ComponentPartial>>::info(&mut parser) {
        Ok((partial, _elements)) => {
            // 基于partial解析结果生成completions
            generate_completions_from_partial(&partial, col)
        }
        Err(_) => {
            // 如果解析失败，提供默认的completions
            vec!["R".to_string(), "C".to_string(), "L".to_string()]
        }
    };

    // 4. 构造 CompletionItem 列表
    let items: Vec<CompletionItem> = completions
        .into_iter()
        .map(|label| {
            let insert_text = if label.starts_with("just test") {
                // 对于测试文本，只取实际有用的建议
                "1".to_string()
            } else {
                label.clone()
            };

            CompletionItem {
                label,
                kind: Some(CompletionItemKind::TEXT),
                detail: Some("SPICE Component".to_string()),
                documentation: Some(Documentation::String("SPICE电路元件".to_string())),
                insert_text: Some(insert_text),
                insert_text_format: Some(InsertTextFormat::PLAIN_TEXT),
                ..Default::default()
            }
        })
        .collect();

    // 5. 返回 CompletionResponse
    // 记录响应内容用于调试
    // client.log_message(MessageType::INFO, &format!("Sending {} completion items", items.len())).await;
    Ok(Some(CompletionResponse::Array(items)))
}

fn generate_completions_from_partial(partial: &ComponentPartial, cursor_pos: usize) -> Vec<String> {
    let mut completions = Vec::new();

    match partial {
        ComponentPartial::R(r_partial) => {
            // 根据光标位置确定应该补全什么
            match cursor_pos {
                0..=2 => {
                    // 光标在组件名称位置
                    if r_partial.name.is_none() {
                        completions.push("R1".to_string());
                        completions.push("R2".to_string());
                        completions.push("R3".to_string());
                    }
                }
                3..=5 => {
                    // 光标在第一个节点位置
                    if r_partial.node1.is_none() {
                        completions.push(
                            "just test,the feature should fetech node from symbol table"
                                .to_string(),
                        );
                        completions.push("1".to_string());
                        completions.push("IN".to_string());
                        completions.push("VCC".to_string());
                    }
                }
                6..=8 => {
                    // 光标在第二个节点位置
                    if r_partial.node2.is_none() {
                        completions.push(
                            "just test,the feature should fetech node from symbol table"
                                .to_string(),
                        );
                        completions.push("0".to_string());
                        completions.push("GND".to_string());
                        completions.push("OUT".to_string());
                    }
                }
                9.. => {
                    // 光标在值位置
                    if r_partial.value.is_none() {
                        completions.push("1k".to_string());
                        completions.push("10k".to_string());
                        completions.push("100".to_string());
                    }
                }
            }
        }

        ComponentPartial::C(c_partial) => {
            // 类似地处理电容组件
            match cursor_pos {
                0..=2 => {
                    // 光标在组件名称位置
                    if c_partial.name.is_none() {
                        completions.push("C1".to_string());
                        completions.push("C2".to_string());
                    }
                }
                3..=5 => {
                    // 光标在第一个节点位置
                    if c_partial.node1.is_none() {
                        completions.push(
                            "just test,the feature should fetech node from symbol table"
                                .to_string(),
                        );
                        completions.push("1".to_string());
                        completions.push("IN".to_string());
                    }
                }
                6..=8 => {
                    // 光标在第二个节点位置
                    if c_partial.node2.is_none() {
                        completions.push(
                            "just test,the feature should fetech node from symbol table"
                                .to_string(),
                        );
                        completions.push("0".to_string());
                        completions.push("GND".to_string());
                    }
                }
                9.. => {
                    // 光标在值位置
                    if c_partial.value.is_none() {
                        completions.push("1uF".to_string());
                        completions.push("10uF".to_string());
                    }
                }
            }
        }

        ComponentPartial::L(l_partial) => {
            // L组件格式: L<name> <node1> <node2> [model] <value> [IC=<initial value>]
            match cursor_pos {
                0..=2 => {
                    // 光标在组件名称位置
                    if l_partial.name.is_none() {
                        completions.push("L1".to_string());
                        completions.push("L2".to_string());
                        completions.push("L3".to_string());
                    }
                }
                3..=5 => {
                    // 光标在第一个节点位置
                    if l_partial.node1.is_none() {
                        completions.push("1".to_string());
                        completions.push("2".to_string());
                        completions.push("IN".to_string());
                    }
                }
                6..=8 => {
                    // 光标在第二个节点位置
                    if l_partial.node2.is_none() {
                        completions.push("0".to_string());
                        completions.push("GND".to_string());
                        completions.push("OUT".to_string());
                    }
                }
                9..=12 => {
                    // 光标在模型或值位置
                    if l_partial.model.is_none() || l_partial.model.as_ref().unwrap().is_none() {
                        completions.push("LMOD".to_string());
                        completions.push("LMODEL".to_string());
                    }
                    if l_partial.value.is_none() {
                        completions.push("1uH".to_string());
                        completions.push("10uH".to_string());
                        completions.push("100nH".to_string());
                        completions.push("1mH".to_string());
                    }
                }
                13.. => {
                    // 光标在参数位置
                    if l_partial.params.is_none() {
                        completions.push("IC=0".to_string());
                        completions.push("IC=1A".to_string());
                    }
                }
            }
        }

        // 如果没有识别出具体的组件类型，提供基本的组件类型补全
        _ => {
            // 根据光标位置提供组件类型建议
            if cursor_pos <= 1 {
                completions.push("R".to_string()); // 电阻
                completions.push("C".to_string()); // 电容
                completions.push("L".to_string()); // 电感
                completions.push("V".to_string()); // 电压源
                completions.push("I".to_string()); // 电流源
            } else {
                // 如果不是在开始位置，提供一些通用的节点名
                completions.push("1".to_string());
                completions.push("2".to_string());
                completions.push("0".to_string());
                completions.push("GND".to_string());
                completions.push("VCC".to_string());
                completions.push("IN".to_string());
                completions.push("OUT".to_string());
            }
        }
    }

    completions
}

#[test]
fn test_partial_parse_resistor() {
    // 测试部分解析一个不完整的电阻组件定义
    let tokens = vec![
        Atom::from("R1"),
        Atom::from("N1"),
        Atom::from("N2"),
        // 注意：缺少电阻值
    ];

    let mut parser = SpiceLineParser::new(&tokens);

    // 使用info方法进行部分解析
    let result = PartialParse::<ComponentPartial>::info(&mut parser);
    assert!(result.is_ok());
    print!("result {:?}", result);

    //let (partial, elements) = result.unwrap();

    // 验证已解析的部分
    //   assert_eq!(partial.name, Some(Name(Atom::from("R1"))));
    //   assert_eq!(partial.node1, Some(Node(Atom::from("1"))));
    //   assert_eq!(partial.node2, Some(Node(Atom::from("2"))));
    //   assert_eq!(partial.value, None); // 未提供，所以是None

    //   // 验证元素列表不为空
    //   assert!(!elements.is_empty());
}

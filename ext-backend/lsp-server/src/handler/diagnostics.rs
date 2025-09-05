use crate::state::{DocumentState, SharedServerState};
use crate::symbol_info::table::SymbolTable;
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


///英文直输：通常每键一次 didChange
/// 粘贴/块替换：通常一次 didChange，content_changes 里 1 个变更，但 text 可能包含很多字符/多行。
pub async fn on_did_change(
    client: &Client,
    state: SharedServerState,
    params: DidChangeTextDocumentParams,
) {
    let uri = params.text_document.uri.clone();

    // 轻量变更摘要（不持锁）：仅数量与首条变更概况
    {
        let msg = if let Some(first) = params.content_changes.first() {
            match &first.range {
                Some(r) => format!(
                    "did_change: n={} first=range({}:{})-({}:{}) text_len={}",
                    params.content_changes.len(),
                    r.start.line,
                    r.start.character,
                    r.end.line,
                    r.end.character,
                    first.text.len()
                ),
                None => format!(
                    "did_change: n={} first=FULL text_len={}",
                    params.content_changes.len(),
                    first.text.len()
                ),
            }
        } else {
            "did_change: n=0".to_string()
        };
        client.log_message(MessageType::INFO, msg).await;
    }

    // 使用增量同步，处理多个变更
    let mut s = state.lock().await;
    if let Some(doc) = s.documents.get_mut(&uri) {
        if let Some(first) = params.content_changes.first() {
            match &first.range {
                None => {
                    // FULL：直接替换整个文档
                    doc.text = first.text.clone();
                }
                Some(_r) => {
                    // INCREMENTAL：按需求改为全量 → 从磁盘加载整文件
                    if let Ok(path) = uri.to_file_path() {
                        if let Ok(content) = std::fs::read_to_string(&path) {
                            doc.text = content;
                        }
                    }
                    // 若读取失败，则不修改 doc.text，保持原内容
                }
            }
        }
        // 标记需要重新解析（若你有标志位可在此设置）
    }
    drop(s); // 释放锁

    // 轻量结果日志（不持锁）：文档长度与行数
    {
        let s = state.lock().await;
        if let Some(doc) = s.documents.get(&uri) {
            client
                .log_message(
                    MessageType::INFO,
                    format!(
                        "did_change: updated len={} lines={}",
                        doc.text.len(),
                        doc.text.lines().count()
                    ),
                )
                .await;
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
            // 预先构建符号表与统计信息（锁外执行，避免锁内重活）
            let instr_count = program.instructions.len();
            let symbols_built = SymbolTable::build_from_ast(uri.clone(), program.clone());
            let symbol_count = symbols_built.table.len();

            // 缓存 AST & 符号表（仅内存写入在锁内）
            {
                let mut s = state.lock().await;
                if let Some(doc) = s.documents.get_mut(&uri) {
                    doc.ast = Some(program);
                    doc.symbols = Some(symbols_built);
                }
            }

            // 清空诊断（或基于 AST 生成真正的诊断）
            //client.publish_diagnostics(uri, vec![], None).await;

            // 成功日志（锁外）：确认 AST 与符号表已构建
            client
                .log_message(
                    MessageType::INFO,
                    format!(
                        "reparse: AST ok, symbols built: {} (instructions: {})",
                        symbol_count, instr_count
                    ),
                )
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


/// 满足任一条件即全量：
/// 1) 有任意一个 change 无 range（全量）
/// 2) 任意一个 change 跨行
/// 3) 任意一个 change 的 text 含换行
/// 4) 一次 didChange 携带多个 change（多选区/多光标批量）




fn incremental_change(text: &mut String, range: &Range, new_text: &str) -> (usize, String) {
    // 拆分原文为行（不保留换行符）
    let lines: Vec<&str> = text.split('\n').collect();

    let start_line_idx = range.start.line as usize;
    let end_line_idx = range.end.line as usize;

    // 起始行左半段
    let left = if start_line_idx < lines.len() {
        let line = lines[start_line_idx];
        let ch = range.start.character as usize;
        line.chars().take(ch).collect::<String>()
    } else {
        String::new()
    };

    // 结束行右半段
    let right = if end_line_idx < lines.len() {
        let line = lines[end_line_idx];
        let ch = range.end.character as usize;
        line.chars().skip(ch).collect::<String>()
    } else {
        String::new()
    };

    // new_text 按行拆分
    let new_parts: Vec<&str> = new_text.split('\n').collect();

    // 计算“变更后起始行”的内容
    let new_start_line = if new_parts.len() == 1 {
        // 单行替换：起始行 = 左半段 + new_text + 右半段（若跨行，右半段来自结束行）
        format!("{}{}{}", left, new_parts[0], right)
    } else {
        // 多行替换：起始行 = 左半段 + new_text 第一行
        format!("{}{}", left, new_parts[0])
    };

    // 组装新文本
    let mut result = String::new();

    // 1) 起始行之前的完整行
    if start_line_idx > 0 {
        result.push_str(&lines[..start_line_idx].join("\n"));
        result.push('\n');
    }

    // 2) 写入起始行（变更后的）
    result.push_str(&new_start_line);

    // 3) 若 new_text 有多行，中间行直接写入
    if new_parts.len() > 1 {
        // 中间的新行（去掉首、尾）
        for mid in &new_parts[1..new_parts.len() - 1] {
            result.push('\n');
            result.push_str(mid);
        }
        // 最后一行 + 结束行右半段
        result.push('\n');
        result.push_str(new_parts.last().unwrap());
        result.push_str(&right);
    }

    // 4) 结束行之后的原始行
    if end_line_idx + 1 < lines.len() {
        result.push('\n');
        result.push_str(&lines[end_line_idx + 1..].join("\n"));
    }

    // 回写
    *text = result;

    // 返回：变更后起始行号与该行文本
    (start_line_idx, new_start_line)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_line_insert() {
        let mut text = String::from("R1 1 2 1k\nC1 2 3 1uF");
        // 在第1行(0-based)第8列插入"0"，将 1k -> 10k
        let range = Range {
            start: Position::new(0, 8),
            end: Position::new(0, 8),
        };
        let (line_idx, new_line) = incremental_change(&mut text, &range, "0");
        assert_eq!(line_idx, 0);
        assert_eq!(new_line, "R1 1 2 10k");
        assert_eq!(text, "R1 1 2 10k\nC1 2 3 1uF");
    }

    #[test]
    fn test_single_line_delete() {
        let mut text = String::from("R1 1 2 1k\nC1 2 3 1uF");
        // 删除第1行(0-based)第8..10列的"1k"
        let range = Range {
            start: Position::new(0, 7),
            end: Position::new(0, 9),
        };
        let (line_idx, new_line) = incremental_change(&mut text, &range, "");
        assert_eq!(line_idx, 0);
        assert_eq!(new_line, "R1 1 2 ");
        assert_eq!(text, "R1 1 2 \nC1 2 3 1uF");
    }

    #[test]
    fn test_insert_newline_within_line() {
        let mut text = String::from("R1 1 2 1k\nC1 2 3 1uF");
        // 在第1行的末尾插入换行与内容
        let range = Range {
            start: Position::new(0, 10),
            end: Position::new(0, 10),
        };
        let (line_idx, new_line) = incremental_change(&mut text, &range, "\nR2 2 3 2k");
        assert_eq!(line_idx, 0);
        assert_eq!(new_line, "R1 1 2 1k");
        assert_eq!(text, "R1 1 2 1k\nR2 2 3 2k\nC1 2 3 1uF");
    }

    #[test]
    fn test_multi_line_replace_spanning() {
        let mut text = String::from("R1 1 2 1k\nC1 0 0 1uF\nL1 3 0 1mH");
        // 用两行文本替换从 (0,8) 到 (1,8) 区间
        // 为了使最后结果合理，这里让 new_text 的最后一行带有空格，拼上 right 后得到原行尾部
        let range = Range {
            start: Position::new(0, 7),
            end: Position::new(1, 7),
        };
        let (line_idx, new_line) = incremental_change(&mut text, &range, "10k\nC1 2 3 ");
        assert_eq!(line_idx, 0);
        assert_eq!(new_line, "R1 1 2 10k");
        // 结束行右半段来自 line1 原文从列8开始，即 "1uF"
        assert_eq!(text, "R1 1 2 10k\nC1 2 3 1uF\nL1 3 0 1mH");
    }
}
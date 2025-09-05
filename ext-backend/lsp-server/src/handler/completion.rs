use crate::state::SharedServerState;
use crate::symbol_info::symbol;
use crate::symbol_info::table::SymbolTable;
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
    let uri = params.text_document_position.text_document.uri.clone();

    // 入口轻量日志，便于确认请求是否到达本函数
    client
        .log_message(
            MessageType::INFO,
            &format!(
                "completion request at {:?}",
                params.text_document_position.clone()
            ),
        )
        .await;

    // 2. 获取文档内容
    let (maybe_source, existed) = {
        let s = state.lock().await;
        let opt = s.documents.get(&uri).cloned();
        (opt, s.documents.contains_key(&uri))
    };

    //let symbol = maybe_source.unwrap().symbols.unwrap();
    
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

    // 兜底：如果文本依然为空，安全返回
    if source.text.trim().is_empty() {
        client
            .log_message(
                MessageType::INFO,
                &format!("completion: no text available for {:?}", uri),
            )
            .await;
        return Ok(Some(CompletionResponse::Array(vec![])));
    }


    client
        .log_message(
            MessageType::INFO,
            &format!(
                "text : {}",
               source.text
            ),
        )
        .await;


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
                "completion safe_line={} total_lines={} extracted='{}' len={}",
                safe_line, total_lines, line_text,line_text.len()
            ),
        )
        .await;



    let mut items: Vec<CompletionItem> = Vec::new();

    if line_text.len() < 3 {
        let cmp_name = generate_component_completions(line_text);

         client
        .log_message(
            MessageType::INFO,
            &format!(
                "completion name={}",
                cmp_name.len()
            ),
        )
        .await;

        for name in cmp_name{
            let cmp_item = CompletionItem {
                label: name.clone(),
                kind: Some(CompletionItemKind::TEXT),
                detail: Some("SPICE Component".to_string()),
                documentation: Some(Documentation::String("SPICE电路元件".to_string())),
                insert_text: Some(name),
                insert_text_format: Some(InsertTextFormat::PLAIN_TEXT),
                ..Default::default()
            };
            items.push(cmp_item);
        }
        
    }else if line_text.len() >= 3 && col == 3 {
        if let Some((label, snippet)) = generate_snippet(line_text, source.symbols.as_ref()) {
            items.push(CompletionItem {
                label: label.to_string(),
                kind: Some(CompletionItemKind::SNIPPET),
                insert_text: Some(snippet),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                ..Default::default()
            })
        }
    }

    

    Ok(Some(CompletionResponse::Array(items)))
}


fn generate_snippet(str:&str,symbols: Option<&SymbolTable>,) -> Option<(String, String)> {

    let names: Vec<String> = symbols
                            .map(|s| s.get_node_names())
                            .unwrap_or_default();

     // ${2|n1,n2,n3|}
    let node_choices_2 = if names.is_empty() {
        "${2|N1,N2,N3|}".to_string()
    } else {
        format!("${{2|{}|}}", names.join(","))
    };

    // ${3|n1,n2,n3|}
    let node_choices_3 = if names.is_empty() {
        "${3|N1,N2,N3|}".to_string()
    } else {
        format!("${{3|{}|}}", names.join(","))
    };

    //let unit_choices = "${5|Ω,kΩ,MΩ|}".to_string();
    match str.chars().next() {
        Some('R') =>{ 
            let snippet = node_choices_2.clone() + " "                          // (+) node
                + &node_choices_3 + " "                                         // (-) node
                + "${5:[model]} "                                               // [model name] 可跳过
                + "${6:value} "                                                 // <value>
                + "${7:[TC1]} "                                                 // [TC=<TC1>] 可跳过
                + "${8:[TC2]}";        
            Some((
                "R 模板:R<name> <(+) node> <(-) node> [model name] <value> [TC = <TC1> [,<TC2>]]".to_string(),
                snippet,                                // [,<TC2>]   
            ))
        }
        Some('C') => {
            let snippet = node_choices_2.clone() + " "                          // (+) node
                + &node_choices_3 + " "
                + "${5:[model]} "                                               // [model name] 可跳过
                + "${6:value} "
                + "${7:[IC=<initial value>]}";

            Some((
                "C 模板:C<name> <(+) node> <(-) node> [model name] <value> [IC=<initial value>]".to_string(),
                snippet,
            ))
        }
        Some('L') => {
            let snippet = node_choices_2.clone() + " "                          // (+) node
                + &node_choices_3 + " "
                + "${5:[model]} "                                               // [model name] 可跳过
                + "${6:value} "
                + "${7:[IC=<initial value>]}";
            Some((
                "L 模板: L<name> <(+) node> <(-) node> [model name] <value> [IC=<initial value>]".to_string(),
                snippet,
            ))
        }
        Some('B') => {
            let snippet = node_choices_2.clone() + " "                          // (+) node
                + &node_choices_3 + " "  
                + &node_choices_3 + " "                        // <source node>
                + "${6:model} "                                            // <model name>
                + "[${7:}]";  
            Some((
                "砷化镓 MES 场效应晶体管: <name> <drain node> <gate node> <source node> <model name> [area value]".to_string(),
                snippet,
            ))    
        }

        Some('D') => {
            let snippet = node_choices_2.clone() + " "                          // (+) node
                + &node_choices_3 + " "                          // <source node>
                + "${6:model} "                                            // <model name>
                + "[${7:area value}]";  
            Some((
                "二极管: D<name> <(+) node> <(-) node> <model name> [area value]".to_string(),
                snippet,
            ))    
        }

        Some('E') => {
            let snippet = node_choices_2.clone() + " "                         
                + &node_choices_3 + " "                         
                + "${4:gain value}"  ;  
            Some((
                "电压源: E<name> <(+) node> <(-) node> <gain>".to_string(),
                snippet,
            ))    
        }

        Some('F') => {
            let snippet = node_choices_2.clone() + " "                         
                + &node_choices_3 + " "                         
                + "${4:gain value}"  ;  
            Some((
                "电流控制电流源: F<name> <(+) node> <(-) node> <gain>".to_string(),
                snippet,
            ))    
        }

        Some('G') => {
            let snippet = node_choices_2.clone() + " "                         
                + &node_choices_3 + " "                         
                + "${4:gain value}"  ;  
            Some((
                "电流控制电流源: G<name> <(+) node> <(-) node> <gain>".to_string(),
                snippet,
            ))    
        }

        Some('H') => {
            let snippet = node_choices_2.clone() + " "                         
                + &node_choices_3 + " "                         
                + "${4:gain value}"  ;  
            Some((
                "电流控制电压源: H<name> <(+) node> <(-) node> <gain>".to_string(),
                snippet,
            ))    
        }

        Some('I') => {
            let snippet = node_choices_2.clone() + " "                         
                + &node_choices_3 + " "                         
                + "${3:[dc]} "
                + "${4:[ac]} "
                + "${5:[transient]}"  ;  
            Some((
                "独立电流源: I<name> <node1> <node2> [<dc>] [<ac>] [<transient>]".to_string(),
                snippet,
            ))    
        }
 
        Some('J') => {
            let snippet = node_choices_2.clone() + " "                         
                + &node_choices_3 + " "  
                + &node_choices_3 + " "                       
                + "${3:model} "
                + "${4:[area value]} ";  
            Some((
                "结型场效应晶体管: J<name> <drain node> <gate node> <source node> <model name> [area value]".to_string(),
                snippet,
            ))    
        }

        //todo  L 的智能补全还未完成
        Some('K') => {
            let snippet = node_choices_2.clone() + " "                         
                + &node_choices_3 + " "  
                + &node_choices_3 + " "                       
                + "${3:model} "
                + "${4:[area value]} ";  
            Some((
                "互感器(L 的智能补全还未完成): K<name> <induct1> <induct2> ... <k> [<model> [<size>]]".to_string(),
                snippet,
            ))    
        }

        Some('M') => {
            let snippet = node_choices_2.clone() + " "                         
                + &node_choices_3 + " "  
                + &node_choices_3 + " "
                + &node_choices_3 + " "                       
                + "${3:model} "
                + " [L=${7}]"                     // optional L
                + " [W=${8}]"                     // optional W
                + " [AD=${9}] [AS=${10}]"         // optional AD/AS
                + " [PD=${11}] [PS=${12}]"        // optional PD/PS
                + " [NRD=${13}] [NRS=${14}]"      // optional NRD/NRS
                + " [NRG=${15}] [NRB=${16}]"      // optional NRG/NRB
                + " [M=${17}] [N=${18}]";  
                      
            Some((
                "MOS 场效应晶体管: M<name> <drain node> <gate node> <source node>`
                + <bulk/substrate node> <model name>`
                + [L=<value>] [W=<value>]`
                + [AD=<value>] [AS=<value>]`
                + [PD=<value>] [PS=<value>]`
                + [NRD=<value>] [NRS=<value>]`
                + [NRG=<value>] [NRB=<value>]`Q
                + [M=<value>] [N=<value>]`".to_string(),
                snippet,
            ))    
        }

        Some('Q') => {
            let snippet = node_choices_2.clone() + " "                         
                + &node_choices_3 + " "  
                + &node_choices_3 + " ";  
            Some((
                "双极结型晶体管:Q<name> <collector node> <base node> <emitter node>".to_string(),
                snippet,
            ))    
        }

        Some('S') => {
            let snippet = node_choices_2.clone() + " "                         
                + &node_choices_3 + " "
                + &node_choices_3 + " "
                + &node_choices_3 + " "
                + "${3:model} ";  
            Some((
                "电压控制开关:S<name> <(+) switch node> <(-) switch node>`
                    <(+) controlling node> <(-) controlling node>`
                    <model name>`".to_string(),
                snippet,
            ))    
        }

        Some('T') => {
            let snippet = node_choices_2.clone() + " "                         
                + &node_choices_3 + " "
                + &node_choices_3 + " "
                + &node_choices_3 + " "
                + "${3:model} "
                + " Z0=${7:value}"                     // characteristic impedance
                + " [TD=${8:value}]"                   // optional TD
                + " [F=${9:value} [NL=${10:value}]]"   // optional F / NL
                + " IC=${11:Vnear} ${12:Inear} ${13:Vfar} ${14:Ifar}"; // initial conditions;  
            Some((
                "输电线路:T<name> <A port (+) node> <A port (-) node>`
                     <B port (+) node> <B port (-) node>`
                     [model name]`
                     Z0=<value> [TD=<value>] [F=<value> [NL=<value>]]`
                     IC= <near voltage> <near current> <far voltage> <far current>`".to_string(),
                snippet,
            ))    
        }

        Some('V') => {
            let snippet = node_choices_2.clone() + " "                         
                + &node_choices_3 + " "  
                + "${3:[dc]} "
                + "${4:[ac]} "
                + "${5:[transient]}";  
            Some((
                "独立电压源:V<name> <node1> <node2> [<dc>] [<ac>] [<transient>]".to_string(),
                snippet,
            ))    
        }

        Some('W') => {
            let snippet = node_choices_2.clone() + " "                         
                + &node_choices_3 + " "  
                + " ${4:Vctrl}"              // controlling voltage device name
                + " ${5:model}";             // model name;  
            Some((
                "电流控制开关:W<name> <(+) switch node> <(-) switch node> <controlling V device name> <model name>".to_string(),
                snippet,
            ))    
        }

        // todo 还没写 X
         Some('X') => {
            let snippet = node_choices_2.clone() + " "                         
                + &node_choices_3 + " "  
                + " ${4:Vctrl}"              // controlling voltage device name
                + " ${5:model}";             // model name;  
            Some((
                "调用子电路:X<name> [node]* <subcircuit name> [PARAM: <<name> = <value>>*]".to_string(),
                snippet,
            ))    
        }

        _ => None,
    }
    

}

fn generate_component_completions(str:&str) -> Vec<String>{
    let mut completions = Vec::new();
    match str.chars().next() {
        Some('R') => {
            completions.push("R1".to_string());
            completions.push("R2".to_string());
            completions.push("R3".to_string());
        }
        Some('C') => {
            completions.push("C1".to_string());
            completions.push("C2".to_string());
            completions.push("C3".to_string());
        }
        Some('L') => {
            completions.push("L1".to_string());
            completions.push("L2".to_string());
            completions.push("L3".to_string());
        }
        _ => {
            completions.push("R".to_string()); // 电阻
            completions.push("C".to_string()); // 电容
            completions.push("L".to_string()); // 电感
            completions.push("V".to_string()); // 电压源
            completions.push("I".to_string()); // 电流源
        } 
        
    }

    completions

}



fn generate_completions_from_partial(
    partial: &ComponentPartial,
    cursor_pos: usize,
    symbols: Option<&SymbolTable>,
) -> Vec<String>{
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
                    if r_partial.node1.is_none() {
                        let names: Vec<String> = symbols
                            .map(|s| s.get_node_names())
                            .unwrap_or_default();
                        if names.is_empty() {
                            completions.extend(["1","IN","VCC"].into_iter().map(String::from));
                        } else {
                            completions.extend(names);
                        }
                    }
                }
                6..=8 => {
                    // 光标在第二个节点位置
                    if r_partial.node2.is_none() {
                       let names: Vec<String> = symbols
                           .map(|s| s.get_node_names())
                           .unwrap_or_default();
                        if names.is_empty() {
                            completions.extend(["1","IN","VCC"].into_iter().map(String::from));
                        } else {
                            completions.extend(names);
                        }   
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
                       let names: Vec<String> = symbols
                           .map(|s| s.get_node_names())
                           .unwrap_or_default();
                        if names.is_empty() {
                            completions.extend(["n1","n2","n3"].into_iter().map(String::from));
                        } else {
                            completions.extend(names);
                        }   
                    }
                }
                6..=8 => {
                    // 光标在第二个节点位置
                    if c_partial.node2.is_none() {
                       let names: Vec<String> = symbols
                           .map(|s| s.get_node_names())
                           .unwrap_or_default();
                        if names.is_empty() {
                            completions.extend(["n1","n2","n3"].into_iter().map(String::from));
                        } else {
                            completions.extend(names);
                        }   
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
                      let names: Vec<String> = symbols
                          .map(|s| s.get_node_names())
                          .unwrap_or_default();
                        if names.is_empty() {
                            completions.extend(["n1","n2","n3"].into_iter().map(String::from));
                        } else {
                            completions.extend(names);
                        }   
                    }
                }
                6..=8 => {
                    // 光标在第二个节点位置
                    if l_partial.node2.is_none() {
                       let names: Vec<String> = symbols
                           .map(|s| s.get_node_names())
                           .unwrap_or_default();
                        if names.is_empty() {
                            completions.extend(["n1","n2","n3"].into_iter().map(String::from));
                        } else {
                            completions.extend(names);
                        }   
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

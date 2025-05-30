use std::{clone, collections::HashMap};
use once_cell::sync::Lazy;
//use spice_parser_core::ast::{Program, try_parse_program};
use spice_parser_core::{ast::{component, expression::Text, Atom, Name}, grammar::*, parse::ExposeNodes, *};
use std::sync::Mutex;
use lsp_types::{Url,*};
use std::fs;
use std::fs::File;
 use std::io::Write;
//use lsp_types::Url;


//#[derive(Serialize, Deserialize, Debug)]
#[derive(Debug)]
struct SymbolInfo {
    name: String,
    uri: Url,// 所在文件
    range: Range,       // 精确的位置
    kind: SpiceSymbolKind,   // 函数、变量、结构体等
}
//type SymbolTable = HashMap<String, Vec<SymbolInfo>>; // 一个符号可能有多个定义（重载等）
#[derive(Debug)]
pub enum SpiceSymbolKind {
    Component,       // 组件类型，例如 R、C、L、V...
    Command,         // 分析命令，如 AC、DC、TRAN
    SubCircuit,      // 子电路（XComponent）引用的子电路名
    Model,           // 模型引用（例如 diode model）
    CircuitName,     // 主电路名（Program.name）
}

static GLOBAL_ASTS: Lazy<Mutex<HashMap<String, ast::Program>>> = Lazy::new(|| Mutex::new(HashMap::new())); //> = Mutex::new(HashMap::new());
static GLOBAL_SYMBOLTABLE: Lazy<Mutex<HashMap<String,Vec<SymbolInfo>>>> = Lazy::new(|| Mutex::new(HashMap::new()));

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

fn update_ast(content: &str, file_uri: Url) {
    let result = try_parse_program(content);
    let mut asts = GLOBAL_ASTS.lock().unwrap();
   
    match result {
        Ok(program) => {
            log::info!("解析出的ast:{:?}", program);
            log::info!("解析成功！");
            log::info!("程序名称: {:?}", program.name);
            log::info!("指令数量: {}", program.instructions.len());
            asts.insert(file_uri.into_string(), program);
        }
        Err(e) => {
            log::error!("解析失败: {:?}", e);
        }
    }
}

pub fn offset_to_line_col(source: &str, offset: usize) -> (u32, u32) {
    let mut line = 0;
    let mut col = 0;
    let mut current_offset = 0;

    for (idx, ch) in source.char_indices() {
        if current_offset == offset {
            break;
        }
        if ch == '\n' {
            line += 1;
            col = 0;
        } else {
            col += 1;
        }
        current_offset += ch.len_utf8();
    }

    (line, col)
}

fn build_symbol_table(ast: &ast::Program, file_uri: Url, content: &str) {
    log::info!("开始构建符号表");
    
    // 创建一个新的符号表
    let mut new_table = HashMap::new();
    log::info!("创建新的符号表完成");
    
    // 处理电路名
    if let Some(name) = &ast.name {
        log::info!("处理电路名: {:?}", name);
        let (start_line, start_col) = offset_to_line_col(content, name.0.column.0);
        let (end_line, end_col) = offset_to_line_col(content, name.0.column.1);
        let mut vec = Vec::new();
        vec.push(SymbolInfo {
            name: name.0.raw.clone(),
            uri: file_uri.clone(),
            range: Range {
                start: Position {
                    line: start_line,
                    character: start_col,
                },
                end: Position {
                    line: end_line,
                    character: end_col,
                },
            },
            kind: SpiceSymbolKind::CircuitName,
        });
        new_table.insert(name.0.raw.clone(), vec);
        log::info!("电路名处理完成");
    }

    log::info!("开始处理指令，数量: {}", ast.instructions.len());
    for (index, instr) in ast.instructions.iter().enumerate() {
        log::info!("处理第 {} 个指令", index + 1);
        match instr {
            ast::Instruction::Component(component) => {
                log::info!("处理组件: {:?}", component);
                let (com_name, column) = match component {
                    spice_parser_core::ast::component::Component::R(comp) => {
                        log::info!("处理电阻组件");
                        (comp.name.0.raw.clone(), comp.name.0.column)
                    },
                    spice_parser_core::ast::component::Component::L(comp) => {
                        log::info!("处理电感组件");
                        (comp.name.0.raw.clone(), comp.name.0.column)
                    },
                    spice_parser_core::ast::component::Component::V(comp) => {
                        log::info!("处理电压源组件");
                        (comp.name.0.raw.clone(), comp.name.0.column)
                    },
                    _ => {
                        log::warn!("未处理的组件类型: {:?}", component);
                        ("UNKNOWN".to_string(), (0, 0))
                    }
                };

                log::info!("组件名称: {}, 位置: {:?}", com_name, column);
                let (start_line, start_col) = offset_to_line_col(content, column.0);
                let (end_line, end_col) = offset_to_line_col(content, column.1);
                log::info!("组件位置: 行={}, 列={} 到 行={}, 列={}", start_line, start_col, end_line, end_col);

                let symbol_info = SymbolInfo {
                    name: com_name.clone(),
                    uri: file_uri.clone(),
                    range: Range {
                        start: Position {
                            line: start_line,
                            character: start_col,
                        },
                        end: Position {
                            line: end_line,
                            character: end_col,
                        },
                    },
                    kind: SpiceSymbolKind::Component,
                };

                new_table.entry(com_name.clone())
                    .or_insert_with(Vec::new)
                    .push(symbol_info);
                log::info!("组件符号信息已添加到表");

                // 处理节点
                log::info!("开始处理组件节点");
                for node in component.nodes() {
                    log::info!("处理节点: {:?}", node);
                    let name = node.0.raw.clone();
                    let (start_line, start_col) = offset_to_line_col(content, node.0.column.0);
                    let (end_line, end_col) = offset_to_line_col(content, node.0.column.1);
                    log::info!("节点位置: 行={}, 列={} 到 行={}, 列={}", start_line, start_col, end_line, end_col);

                    let symbol_info = SymbolInfo {
                        name: name.clone(),
                        uri: file_uri.clone(),
                        range: Range {
                            start: Position {
                                line: start_line,
                                character: start_col,
                            },
                            end: Position {
                                line: end_line,
                                character: end_col,
                            },
                        },
                        kind: SpiceSymbolKind::Component,
                    };

                    new_table.entry(name.clone())
                        .or_insert_with(Vec::new)
                        .push(symbol_info);
                    log::info!("节点符号信息已添加到表");
                }
                log::info!("组件节点处理完成");
            },
            ast::Instruction::Command(cmd) => {
                log::info!("处理命令: {:?}", cmd);
                let (cmd_name, column) = match cmd {
                    spice_parser_core::ast::command::Command::Model(cmd) => {
                        log::info!("处理 Model 命令");
                        (cmd.name.0.raw.clone(), cmd.name.0.column.clone())
                    },
                    spice_parser_core::ast::command::Command::Subckt(cmd) => {
                        log::info!("处理 Subckt 命令");
                        (cmd.name.0.raw.clone(), cmd.name.0.column.clone())
                    },
                    spice_parser_core::ast::command::Command::Global(cmd) => {
                        log::info!("处理 Global 命令");
                        (cmd.node.0.raw.clone(), cmd.node.0.column.clone())
                    },
                    spice_parser_core::ast::command::Command::Inc(cmd) => {
                        log::info!("处理 Inc 命令");
                        (cmd.filename.0.raw.clone(), cmd.filename.0.column.clone())
                    },
                    spice_parser_core::ast::command::Command::Lib(cmd) => {
                        log::info!("处理 Lib 命令");
                        let default = Text(Atom::new("UNKNOWN", (0, 0)));
                        let atom = cmd.filename.clone().unwrap_or(default.clone()).0;
                        (atom.raw, atom.column)
                    },
                    spice_parser_core::ast::command::Command::Ends(cmd) => {
                        log::info!("处理 Ends 命令");
                        let default = Name(Atom::new("UNKNOWN", (0, 0)));
                        let atom = cmd.name.clone().unwrap_or(default.clone()).0;
                        (atom.raw, atom.column)
                    },
                    spice_parser_core::ast::command::Command::Tran(cmd) => {
                        log::info!("处理 Tran 命令");
                        // 对于 Tran 命令，我们使用命令本身作为符号
                        (".TRAN".to_string(), (0, 0)) // 这里需要根据实际情况修改
                    },
                    _ => {
                        log::warn!("未处理的命令类型: {:?}", cmd);
                        ("UNKNOWN".to_string(), (0, 0))
                    }
                };

                log::info!("命令名称: {}, 位置: {:?}", cmd_name, column);
                let (start_line, start_col) = offset_to_line_col(content, column.0);
                let (end_line, end_col) = offset_to_line_col(content, column.1);
                log::info!("命令位置: 行={}, 列={} 到 行={}, 列={}", start_line, start_col, end_line, end_col);

                let symbol_info = SymbolInfo {
                    name: cmd_name.clone(),
                    uri: file_uri.clone(),
                    range: Range {
                        start: Position {
                            line: start_line,
                            character: start_col,
                        },
                        end: Position {
                            line: end_line,
                            character: end_col,
                        },
                    },
                    kind: SpiceSymbolKind::Command,
                };

                new_table.entry(cmd_name.clone())
                    .or_insert_with(Vec::new)
                    .push(symbol_info);
                log::info!("命令符号信息已添加到表");
            }
        }
    }
    
    
    // 打印完整的符号表内容
    // log::info!("当前符号表内容:");
    // for (symbol_name, symbol_infos) in &new_table {
    //     log::info!("符号名: {}", symbol_name);
    //     for (index, info) in symbol_infos.iter().enumerate() {
    //         log::info!("  定义 #{}:", index + 1);
    //         log::info!("    名称: {}", info.name);
    //         log::info!("    类型: {:?}", info.kind);
    //         log::info!("    位置: 行={}, 列={} 到 行={}, 列={}", 
    //             info.range.start.line,
    //             info.range.start.character,
    //             info.range.end.line,
    //             info.range.end.character);
    //     }
    // }
            
    // 最后一次性更新全局符号表
    {
        log::info!("获取全局符号表锁");
        let mut table = GLOBAL_SYMBOLTABLE.lock().unwrap();
        log::info!("获取锁成功，开始更新");
        *table = new_table;
        log::info!("更新完成");
    }
    
    log::info!("符号表构建完成");
}

fn read_file_content_from_url(url: &Url) -> Result<String, std::io::Error> {
    // 只支持 file:// 协议
    if url.scheme() != "file" {
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "Only file:// URLs are supported"));
    }

    // 转换成 PathBuf
    let path = url.to_file_path().map_err(|_| {
        std::io::Error::new(std::io::ErrorKind::InvalidInput, "Invalid file URL path")
    })?;

    // 读取文件内容
    fs::read_to_string(path)
}

pub fn update(file_uri: Url) {
    log::info!("_____________________-开始更新文件: {:?}", file_uri);
    
    let content = match read_file_content_from_url(&file_uri) {
        Ok(content) => {
            log::info!("成功读取文件内容，长度: {}", content.len());
            content
        },
        Err(e) => {
            log::error!("读取文件失败: {:?}", e);
            return;
        },
    };

    // 使用一个作用域来确保锁的释放
    let ast = {
        log::info!("准备更新 AST");
        let result = try_parse_program(&content);
        let mut asts = GLOBAL_ASTS.lock().unwrap();
        
        match result {
            Ok(program) => {
                log::info!("解析成功，准备插入 AST");
                asts.insert(file_uri.to_string(), program);
                log::info!("AST 插入完成");
                
                match asts.get(&file_uri.to_string()) {
                    Some(ast) => {
                        log::info!("成功获取 AST");
                        ast.clone()
                    },
                    None => {
                        log::error!("获取 AST 失败");
                        return;
                    },
                }
            },
            Err(e) => {
                log::error!("解析失败: {:?}", e);
                return;
            }
        }
    };

    log::info!("准备构建符号表");
    build_symbol_table(&ast, file_uri.clone(), &content);
    log::info!("符号表构建完成");
}

pub fn handle_goto_definition(params: TextDocumentPositionParams) -> Option<Vec<Location>> {
    log::info!("开始处理跳转定义请求");
    
    // 先更新文件
    update(params.text_document.uri.clone());
    
    // 获取当前行的文本
    let content = match read_file_content_from_url(&params.text_document.uri) {
        Ok(content) => content,
        Err(e) => {
            log::error!("读取文件失败: {:?}", e);
            return None;
        },
    };
    
    let lines: Vec<&str> = content.lines().collect();
    let line_text = match lines.get(params.position.line as usize) {
        Some(text) => {
            log::info!("当前行文本: {}", text);
            text
        },
        None => {
            log::error!("读取行失败");
            return None;
        },
    };

    // 提取当前光标位置的单词
    let symbol_name = match extract_word(line_text, params.position.character as usize) {
        Some(name) => {
            log::info!("提取的符号名: {}", name);
            name
        },
        None => {
            log::error!("提取单词失败");
            return None;
        },
    };

    // 从符号表中查找定义
    // let location:Option<Vec<Location>> = {
    //     let symbol_table = GLOBAL_SYMBOLTABLE.lock().unwrap();
    //     match symbol_table.get(&symbol_name) {
    //         Some(defs) => {
    //             log::info!("查询正常，找到定义: {:?}", defs);
    //             defs.iter().map(|def| Location {
    //                 uri: def.uri.clone(),
    //                 range: def.range,
    //             }).collect()
    //         },
    //         None => {
    //             log::error!("查找符号表失败，符号名: {}", symbol_name);
    //             None
    //         },
    //     }
    // };

    let locations: Option<Vec<Location>> = {
        let symbol_table = GLOBAL_SYMBOLTABLE.lock().unwrap();
        match symbol_table.get(&symbol_name) {
            Some(defs) => {
                log::info!("查询正常，找到定义: {:?}", defs);
                Some(defs.iter().map(|def| Location {
                    uri: def.uri.clone(),
                    range: def.range,
                }).collect())
            },
            None => {
                log::error!("查找符号表失败，符号名: {}", symbol_name);
                None
            },
        }
    };

    locations
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

    #[test]
    fn test_update_ast_success() {
        let content = "
    RCL

    R1 N2 N3 10.0  
    L1 N2 N1 10.0E-3  
    L2 N3 N4 10.0E-3  
    V1I95 N1 0 AC 0.0 SIN ( 0.0 5.0 1.0E3 0.0 0.0 0.0 )  
    R2 0 N4 10.0  
    C1 N2 N4 10.0E-6  

    .TRAN 0.1M 10M
    .LIB
    .IC V(2)=3 V(2)=4
    .END
";
        let url = Url::parse("file:///test1.ast").unwrap();

        update_ast(content, url.clone());

        // let asts = GLOBAL_ASTS.lock().unwrap();
        // let program = asts.get(&url).expect("Program should be inserted");

        // assert_eq!(program.name, "TestProgram");
        // assert_eq!(program.instructions.len(), 2);
    }
   
     #[test]
    fn test_offset_start_of_text() {
      
        let source = "
        R1 N1 N2
        V1 N3 0
        .TRAN 1m 10m
        ";
        let offset = 0; // "R1 N1 N2\n" = 9 (含 \n)，下一个是第2行开始
        assert_eq!(offset_to_line_col(source, offset), (0, 0));


    }

    #[test]
fn test_build_symbol_table_simple() {
    // 准备测试输入
    let file_uri = Url::parse("file:///test.sp").unwrap();
    let content = "
   RCL

    R1 N2 N3 10.0  
    L1 N2 N1 10.0E-3  
    L2 N3 N4 10.0E-3  
    V1I95 N1 0 AC 0.0 SIN ( 0.0 5.0 1.0E3 0.0 0.0 0.0 )  
    R2 0 N4 10.0  
    C1 N2 N4 10.0E-6  

    .TRAN 0.1M 10M
    .LIB
    .IC V(2)=3 V(2)=4
    .END
    ";

    update_ast(content, file_uri.clone());
   
    // 清空符号表
    GLOBAL_SYMBOLTABLE.lock().unwrap().clear();
    let global_asts = GLOBAL_ASTS.lock().unwrap(); // 保证 MutexGuard 活着
    let ast = global_asts.get(&file_uri.to_string()).unwrap(); // safe unwrap
    // 执行构建
    build_symbol_table(ast, file_uri.clone(), content);

    // 检查 symbol table 中的值
    let table = GLOBAL_SYMBOLTABLE.lock().unwrap();

    print!("{:?}",table);
    // 校验 Subckt 命令
    let sym_list = table.get("test_node").unwrap();
    assert_eq!(sym_list.len(), 1);
    let sym = &sym_list[0];
    assert_eq!(sym.name, "test_node");
    assert_eq!(sym.uri, file_uri);
    //assert_eq!(sym.kind, SpiceSymbolKind::Command);
    assert_eq!(sym.range.start.line, 0); // 根据 offset_to_line_col 预期
    assert_eq!(sym.range.start.character, 8);

    // 校验元件节点 N1
    let sym_list = table.get("N1").unwrap();
    assert_eq!(sym_list.len(), 1);
   // assert_eq!(sym_list[0].kind, SpiceSymbolKind::Component);

    // 校验电路名 main
    //let sym_list = table.get("main").unwrap();
    //assert_eq!(sym_list[0].kind, SpiceSymbolKind::CircuitName);
}


 #[test]
    fn test_update_functionality() {
        use std::path::Path;

       // 你的真实文件路径
    let file_path = Path::new("G:\\lsptest\\extention-demo\\test.txt");

    // 确保文件存在
    assert!(file_path.exists(), "测试文件不存在: {:?}", file_path);

    // 转换为 file:// URL
    let file_uri = Url::from_file_path(file_path).expect("无法将路径转换为 Url");


        // 清空 AST/Symbol 表（如已初始化）
        {
            GLOBAL_ASTS.lock().unwrap().clear();
            GLOBAL_SYMBOLTABLE.lock().unwrap().clear();
        }

        // 调用待测函数
        update(file_uri.clone());

        // 验证 AST 是否更新
        {
            let asts = GLOBAL_ASTS.lock().unwrap();
            assert!(asts.contains_key(&file_uri.to_string()));
        }

        // 验证符号表是否构建
        {
            let symbol_table = GLOBAL_SYMBOLTABLE.lock().unwrap();
            assert!(symbol_table.contains_key("N1")); // 根据上面 test_content 中的符号
            assert!(symbol_table.contains_key("0"));
            //assert!(symbol_table.contains_key("ccb"));
        }
    }


     #[test]
    fn test_handle_goto_definition() {
        use std::path::Path;

       // 你的真实文件路径
    let file_path = Path::new("G:\\lsptest\\extention-demo\\test.txt");

    // 确保文件存在
    assert!(file_path.exists(), "测试文件不存在: {:?}", file_path);

    // 转换为 file:// URL
    let file_uri = Url::from_file_path(file_path).expect("无法将路径转换为 Url");


        // 清空 AST/Symbol 表（如已初始化）
        {
            GLOBAL_ASTS.lock().unwrap().clear();
            GLOBAL_SYMBOLTABLE.lock().unwrap().clear();
        }

        // 调用待测函数
        update(file_uri.clone());
         let params = TextDocumentPositionParams {
            text_document: TextDocumentIdentifier { uri: file_uri.clone() },
            position: Position { line: 2, character: 0 },
        };
        
         // 构造伪造符号表条目
        let def_location = Location {
            uri: file_uri.clone(),
            range: Range {
                start: Position { line: 0, character: 0 },
                end: Position { line: 2, character: 2 },
            },
        };

        // 执行跳转
        let result = handle_goto_definition(params);

        print!("{:?}",result);

        // 断言返回的结果与我们注入的定义一致
        assert_eq!(result, Some(def_location));
    }

}
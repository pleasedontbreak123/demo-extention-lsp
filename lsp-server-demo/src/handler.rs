use std::collections::HashMap;
use spice_parser_core::ast::*;

use lsp_types::*;


#[derive(Serialize, Deserialize, Debug)]
struct SymbolInfo {
    name: String,
    uri: Url,           // 所在文件
    range: Range,       // 精确的位置
    kind: SymbolKind,   // 函数、变量、结构体等
}
type SymbolTable = HashMap<String, Vec<SymbolInfo>>; // 一个符号可能有多个定义（重载等）

static GLOBAL_ASTS: Lazy<Mutex<HashMap<String, Result<Program>>>> = Lazy::new(|| Mutex::new(HashMap::new())); //> = Mutex::new(HashMap::new());

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


fn build_symbol_table(ast: &AstNode, file_uri: Url) -> SymbolTable {
    let mut table = SymbolTable::new(); //动态加载再说吧
    
    table
}





fn handle_goto_definition() -> Option<Location>{

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
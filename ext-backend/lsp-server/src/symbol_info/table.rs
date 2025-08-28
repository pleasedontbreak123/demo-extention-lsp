

use tower_lsp::lsp_types::{Position, Range, Url};
use super::symbol::{Symbol, SpiceSymbolKind};
use spice_parser_core::ast::{Program, Instruction, component::Component, Name};

#[derive(Debug, Clone)]
pub struct SymbolTable{
    pub uri: Url,
    pub table: Vec<Symbol>
}

impl SymbolTable{

    pub fn new(uri: Url) -> Self {
        SymbolTable { uri, table: Vec::new() }
    }

    pub fn build_from_ast(uri: Url, program: Program) -> Self {
        let mut table = SymbolTable::new(uri.clone());

        if let Some(name) = program.name.clone() {
            table.table.push(Self::symbol_from_name(&name, SpiceSymbolKind::CircuitName));
        }

        for ins in &program.instructions {
            Self::collect_node(&uri, ins, &mut table, None);
        }

        table
    }

    fn collect_node(
        _uri: &Url,
        ins: &Instruction,
        table: &mut SymbolTable,
        container: Option<String>,
    ) {
        match ins {
            Instruction::Component(c) => {
                let sym = Self::symbol_from_component(c, container.clone());
                table.table.push(sym);
            }
            Instruction::Command(_cmd) => {
                // 暂不为命令生成符号
            }
        }
    }

    fn symbol_from_component(cmp: &Component, container: Option<String>) -> Symbol {
        let name = Self::component_name(cmp);
        Symbol {
            name: name.0.to_string(),
            range: Self::name_to_range(name),
            kind: SpiceSymbolKind::Component,
            container,
        }
    }

    fn component_name(cmp: &Component) -> &Name {
        use Component::*;
        match cmp {
            B(x) => &x.name,
            C(x) => &x.name,
            D(x) => &x.name,
            E(x) => &x.name,
            F(x) => &x.name,
            G(x) => &x.name,
            H(x) => &x.name,
            I(x) => &x.name,
            J(x) => &x.name,
            K(x) => &x.name,
            L(x) => &x.name,
            M(x) => &x.name,
            Q(x) => &x.name,
            R(x) => &x.name,
            S(x) => &x.name,
            T(x) => &x.name,
            V(x) => &x.name,
            W(x) => &x.name,
            X(x) => &x.name,
            Z(x) => &x.name,
        }
    }

    fn symbol_from_name(name: &Name, kind: SpiceSymbolKind) -> Symbol {
        Symbol {
            name: name.0.to_string(),
            range: Self::name_to_range(name),
            kind,
            container: None,
        }
    }

    fn name_to_range(name: &Name) -> Range {
        let atom = &name.0;
        let line = atom.line as u32;
        let (start, end) = atom.column;
        Range {
            start: Position::new(line, start as u32),
            end: Position::new(line, end as u32),
        }
    }
}

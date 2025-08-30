use std::{
    collections::{BTreeMap, HashMap},
    hash::{Hash, Hasher},
};

use super::symbol::{SpiceSymbolKind, Symbol};
use spice_parser_core::ast::{Instruction, Name, Program, component::Component};
use tower_lsp::lsp_types::{Position, Range, Url};

#[derive(Debug, Clone)]
pub struct OrderedRange {
    pub start: Position,
    pub end: Position,
}

impl From<Range> for OrderedRange {
    fn from(value: Range) -> Self {
        Self {
            start: value.start,
            end: value.end,
        }
    }
}

impl PartialEq for OrderedRange {
    fn eq(&self, other: &Self) -> bool {
        self.start == other.start && self.end == other.end
    }
}

impl Eq for OrderedRange {}

impl Hash for OrderedRange {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.start.line.hash(state);
        self.start.character.hash(state);
        self.end.line.hash(state);
        self.end.character.hash(state);
    }
}

impl PartialOrd for OrderedRange {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match self.start.partial_cmp(&other.start) {
            Some(std::cmp::Ordering::Equal) => {
                self.end.partial_cmp(&other.end)
            }
            ordering => ordering,
        }
    }
}

impl Ord for OrderedRange {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.start.cmp(&other.start) {
            std::cmp::Ordering::Equal => {
                self.end.cmp(&other.end)
            }
            ordering => ordering,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SymbolTable {
    pub uri: Url,
    pub table: HashMap<String, Symbol>,
    pub range: BTreeMap<OrderedRange, String>,
}

impl SymbolTable {
    pub fn new(uri: Url) -> Self {
        SymbolTable {
            uri,
            table: HashMap::new(),
            range: BTreeMap::new(),
        }
    }

    pub fn build_from_ast(uri: Url, program: Program) -> Self {
        let mut table = SymbolTable::new(uri.clone());

        if let Some(name) = program.name.clone() {
            let symbol = Self::symbol_from_name(&name, SpiceSymbolKind::CircuitName);
            table.range.insert(symbol.range.into(), symbol.name.clone());
            table.table.insert(symbol.name.clone(), symbol);
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
                table.range.insert(sym.range.into(), sym.name.clone());
                table.table.insert(sym.name.clone(), sym);
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
            refcnt: 0,
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
            refcnt: 0,
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

    /// 根据给定的位置查找对应的符号名
    pub fn symbol_name_at_position(&self, position: Position) -> Option<String> {
        self.range
            .iter()
            .find(|(range, _)| position >= range.start && position < range.end)
            .map(|(_, name)| name.clone())
    }

    /// 根据给定的位置查找对应的符号
    pub fn symbol_at_position(&self, position: Position) -> Option<&Symbol> {
        self.symbol_name_at_position(position)
            .and_then(|name| self.table.get(&name))
    }
}

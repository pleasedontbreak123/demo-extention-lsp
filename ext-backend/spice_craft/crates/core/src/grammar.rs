use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};

use anyhow::{anyhow, Result};

use crate::{
    ast::{command::Command, component::Component, Instruction, Program},
    try_parse_program,
};

/// PSpice Components.
#[derive(Debug, Default, Clone)]
pub struct PSpiceComponent {}

/// PSpice Sub-circuit.
#[derive(Debug, Default, Clone)]
pub struct PSpiceSubckt {
    components: Vec<Component>,
}

/// PSpice Commands.
///
/// Include and expansion commands will be expanded.

#[derive(Debug, Clone)]
pub struct PSpiceCommand(pub Command);

struct Worker {
    visit: HashSet<PathBuf>,
}

impl Worker {
    fn new() -> Self {
        Self {
            visit: HashSet::new(),
        }
    }

    fn work(&mut self, filename: impl AsRef<Path>) -> Result<Program> {
        let filename = filename.as_ref().to_path_buf();
        if self.visit.contains(&filename) {
            return Err(anyhow!("Recursive include detected: {filename:?}"));
        }

        self.visit.insert(filename.clone());
        let content = fs::read_to_string(&filename).unwrap();

        let mut program = try_parse_program(&content)?;

        let mut instructions = Vec::new();
        for ins in program.instructions {
            match &ins {
                Instruction::Command(cmd) => match cmd {
                    Command::Inc(inc) => {
                        let inc = filename.join(inc.filename.to_string());
                        let mut program = self.work(inc)?;
                        instructions.append(&mut program.instructions);
                    }
                    Command::Lib(lib) => {
                        let lib = match &lib.filename {
                            Some(lib) => filename.join(lib.to_string()),
                            None => filename.with_extension("lib"),
                        };
                        let mut program = self.work(lib)?;
                        instructions.append(&mut program.instructions);
                    }
                    _ => instructions.push(ins),
                },
                _ => instructions.push(ins),
            }
        }
        program.instructions = instructions;

        Ok(program)
    }
}
#[derive(Debug, Default, Clone)]
pub struct PSpice {
    pub name: String,
    pub nodes: Vec<String>,
    pub components: Vec<Component>,
    pub subckt: Vec<PSpiceSubckt>,
    pub commands: Vec<Command>,
}

impl PSpice {
    pub fn read_all_files(filename: impl AsRef<Path>) -> Vec<PathBuf> {
        todo!()
    }

    /// 从文件开始解析
    pub fn try_parse_from_file(filename: impl AsRef<Path>) -> Result<Self> {
        // Parse content first.
        let mut ret = Self::default();
        let mut worker = Worker::new();
        let program = worker.work(filename)?;

        for instruction in program.instructions {
            match instruction {
                Instruction::Command(k) => match k {
                    k => ret.commands.push(k),
                },
                Instruction::Component(k) => match k {
                    k => ret.components.push(k),
                },
            }
        }

        Ok(ret)
    }

    /// 从 stdin 中解析，只能解析出单文件，无法解析多文件
    pub fn try_parse_from_stdin(content: &str) -> Result<Self> {
        let program = try_parse_program(&content)?;
        let mut ret = Self::default();
        for insn in program.instructions {
            match insn {
                Instruction::Command(k) => match k {
                    Command::Inc(inc) => Err(anyhow!(
                        "Requested INC {}, but from stdin.",
                        inc.filename.to_string()
                    ))?,
                    Command::Lib(lib) => Err(anyhow!(
                        "Requested LIB {}, but from stdin.",
                        lib.filename
                            .map_or_else(|| "?".to_owned(), |x| x.to_string())
                    ))?,
                    k => ret.commands.push(k),
                },
                Instruction::Component(k) => match k {
                    k => ret.components.push(k),
                },
            }
        }
        Ok(ret)
    }
}

/// Will not accept `INC` and `LIB` commands.
///
/// Includes are to be expanded before using TryFrom.
impl TryFrom<Program> for PSpice {
    type Error = ();
    fn try_from(value: Program) -> Result<Self, Self::Error> {
        todo!()
    }
}

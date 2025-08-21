use std::{collections::HashSet, fs, path::PathBuf};

use codespan_reporting::{
    diagnostic::{Diagnostic, Label},
    files::SimpleFiles,
    term::{
        self,
        termcolor::{ColorChoice, StandardStream},
        Config,
    },
};
use spice_parser_core::{
    ast::{command::Command, Instruction, Program},
    try_parse_program,
};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "pspice", about = "A simple PSpice parser")]
struct Opt {
    /// Output in json format
    #[structopt(short, long)]
    json: bool,
    /// Pretty output
    #[structopt(short, long)]
    pretty: bool,
    /// .cir file to parse
    file: String,
}

struct Worker {
    files: SimpleFiles<String, String>,
    visit: HashSet<PathBuf>,
}

type PSpiceResult<T> = Result<T, Diagnostic<usize>>;

impl Worker {
    fn new() -> Self {
        Self {
            files: SimpleFiles::new(),
            visit: HashSet::new(),
        }
    }

    fn work(&mut self, filename: PathBuf) -> PSpiceResult<Program> {
        if self.visit.contains(&filename) {
            return Err(Diagnostic::error()
                .with_message(format!("Recursive include detected: {filename:?}")));
        }

        self.visit.insert(filename.clone());
        let content = fs::read_to_string(&filename).unwrap();
        let file_id = self
            .files
            .add(filename.to_string_lossy().to_string(), content.clone());

        let mut program = try_parse_program(&content).map_err(|e| {
            Diagnostic::error().with_message(e.reason).with_labels(
                if let Some((st, ed)) = e.position {
                    vec![Label::primary(file_id, st..ed)]
                } else {
                    Vec::new()
                },
            )
        })?;

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

fn main() {
    let opt = Opt::from_args();
    let mut worker = Worker::new();

    match worker.work(fs::canonicalize(opt.file).unwrap()) {
        Ok(program) => {
            if opt.json {
                let json = if opt.pretty {
                    serde_json::to_string_pretty(&program).unwrap()
                } else {
                    serde_json::to_string(&program).unwrap()
                };

                println!("{}", json);
            }
        }
        Err(e) => {
            let writer = StandardStream::stderr(ColorChoice::Always);
            let config = Config::default();
            term::emit(&mut writer.lock(), &config, &worker.files, &e).unwrap();
        }
    }
}

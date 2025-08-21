use nom::Err;
use serde::Serialize;

use crate::ast::{
    command::{Command, SubcktCommand},
    Atom, Instruction, Name, Node, Program,
};

#[derive(Debug, PartialEq, Serialize)]
pub struct ParseError {
    pub reason: String,
    pub position: Option<(usize, usize)>,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some((line, col)) = self.position {
            write!(
                f,
                "Parse error at line {}, column {}: {}",
                line, col, self.reason
            )
        } else {
            write!(f, "Parse error: {}", self.reason)
        }
    }
}

impl std::error::Error for ParseError {}

// some helper functions (seems useless)
impl ParseError {
    pub fn unexpected_eof(position: Option<(usize, usize)>) -> Self {
        Self {
            reason: format!("Unexpected EOF"),
            position,
        }
    }

    pub fn unexpected(expect: &str, actual: &str, position: Option<(usize, usize)>) -> Self {
        Self {
            reason: format!("Expect `{}', found `{}'", expect, actual),
            position,
        }
    }

    pub fn unknown_suffix(found: &str, accepts: &str, position: Option<(usize, usize)>) -> Self {
        Self {
            reason: format!("Unknown suffix: `{}', only accepts {}", found, accepts),
            position,
        }
    }

    pub fn fallback_position(mut self, position: (usize, usize)) -> Self {
        if self.position.is_none() {
            self.position = Some(position)
        }
        self
    }
}

pub type ParseResult<T> = Result<T, ParseError>;
pub type Token = Atom;

pub trait TryParse<T> {
    fn try_parse(&mut self) -> ParseResult<T>;
}

pub trait ExposeNodes {
    fn nodes(&self) -> Vec<Node>;
}

#[derive(Clone, Debug)]
pub struct Element {
    inner: ElementInfo,
    line: usize,
    character: usize,
}
#[derive(Clone, Debug)]
pub enum ElementInfo {
    Number,
    Node(String),
}

pub trait PartialParse<T> {
    fn try_partial(&mut self) -> ParseResult<T>;
    fn extend(&mut self, src: &mut T) -> ParseResult<()> {
        todo!()
    }
    fn info(&mut self) -> ParseResult<(T, Vec<Element>)>;
}

pub trait TokenStream: Snapshot {
    /// 返回总 Token 数量
    fn len(&self) -> usize;
    /// 返回当前 Token 位置
    fn position(&self) -> usize;
    /// 返回指定位置的 Token
    fn get(&self, index: usize) -> Option<&Token>;
    /// 前进到下一个 Token
    fn consume(&mut self);

    /// 返回当前 Token，如果 EOF 则返回错误
    fn token(&self) -> ParseResult<Token> {
        self.get(self.position())
            .cloned()
            .ok_or_else(|| ParseError::unexpected_eof(None))
    }

    /// 返回下一个位置的 Token，如果 EOF 则返回错误
    fn next_token(&self) -> ParseResult<Token> {
        self.get(self.position() + 1)
            .cloned()
            .ok_or_else(|| ParseError::unexpected_eof(None))
    }

    /// 检测当前 Token 是否匹配（忽略大小写）
    fn matches(&self, matches: &str) -> bool {
        self.get(self.position())
            .map(|token| token.raw.to_ascii_uppercase() == matches.to_ascii_uppercase())
            .unwrap_or(false)
    }

    /// 检测当前 Token 是否匹配（忽略大小写），如果匹配则前进到下一个 Token
    fn matches_consume(&mut self, matches: &str) -> bool {
        if self.matches(matches) {
            self.consume();
            true
        } else {
            false
        }
    }

    /// 检测下一个 Token 是否匹配（忽略大小写）
    fn next_matches(&self, matches: &str) -> bool {
        self.get(self.position() + 1)
            .map(|token| token.raw.to_ascii_uppercase() == matches.to_ascii_uppercase())
            .unwrap_or(false)
    }

    /// 检测是否 EOF
    fn is_eof(&self) -> bool {
        self.position() >= self.len()
    }

    /// 检测下一个是否是 EOF
    fn next_eof(&self) -> bool {
        self.position() + 1 >= self.len()
    }

    /// 断言当前 Token 是否匹配（忽略大小写）
    fn expect(&mut self, token: &str) -> ParseResult<Token> {
        let actual = self.token()?;
        if actual.raw.to_ascii_uppercase() != token.to_ascii_uppercase() {
            return Err(ParseError::unexpected(
                token,
                &actual.raw[..],
                Some(actual.column),
            ));
        }
        self.consume();
        Ok(actual)
    }

    /// 断言不是 EOF
    fn expect_eof(&self) -> ParseResult<()> {
        if self.is_eof() {
            Ok(())
        } else {
            let token = self.token()?;
            Err(ParseError::unexpected(
                "<EOF>",
                &token.raw[..],
                Some(token.column),
            ))
        }
    }
}

pub trait Snapshot {
    type State;
    fn snapshot(&self) -> Self::State;
    fn restore(&mut self, state: Self::State);
}

impl<P> TryParse<usize> for P
where
    P: TokenStream,
{
    fn try_parse(&mut self) -> ParseResult<usize> {
        let token = self.token()?;
        let res = token
            .raw
            .parse()
            .map_err(|_| ParseError::unexpected("<usize>", &token.raw[..], Some(token.column)));
        if res.is_ok() {
            self.consume();
        }
        res
    }
}

impl<T, P> TryParse<Option<T>> for P
where
    P: TryParse<T> + Snapshot,
{
    fn try_parse(&mut self) -> ParseResult<Option<T>> {
        let snapshot = self.snapshot();
        match self.try_parse() {
            Ok(value) => Ok(Some(value)),
            Err(_) => {
                self.restore(snapshot);
                Ok(None)
            }
        }
    }
}

impl<T, P> TryParse<Vec<T>> for P
where
    P: TryParse<T> + Snapshot,
{
    fn try_parse(&mut self) -> ParseResult<Vec<T>> {
        let mut values = Vec::new();
        let mut snapshot = self.snapshot();
        while let Ok(value) = self.try_parse() {
            values.push(value);
            snapshot = self.snapshot();
        }
        self.restore(snapshot);
        Ok(values)
    }
}

impl<P, T1, T2> TryParse<(T1, T2)> for P
where
    P: TryParse<T1> + TryParse<T2>,
{
    fn try_parse(&mut self) -> ParseResult<(T1, T2)> {
        let value1 = self.try_parse()?;
        let value2 = self.try_parse()?;
        Ok((value1, value2))
    }
}

impl<P, T1, T2, T3> TryParse<(T1, T2, T3)> for P
where
    P: TryParse<T1> + TryParse<T2> + TryParse<T3>,
{
    fn try_parse(&mut self) -> ParseResult<(T1, T2, T3)> {
        let value1 = self.try_parse()?;
        let value2 = self.try_parse()?;
        let value3 = self.try_parse()?;
        Ok((value1, value2, value3))
    }
}

impl<P, T1, T2, T3, T4> TryParse<(T1, T2, T3, T4)> for P
where
    P: TryParse<T1> + TryParse<T2> + TryParse<T3> + TryParse<T4>,
{
    fn try_parse(&mut self) -> ParseResult<(T1, T2, T3, T4)> {
        let value1 = self.try_parse()?;
        let value2 = self.try_parse()?;
        let value3 = self.try_parse()?;
        let value4 = self.try_parse()?;
        Ok((value1, value2, value3, value4))
    }
}

pub struct SpiceLineParser<'a> {
    vec: &'a Vec<Token>,
    position: usize,
}

impl<'a> SpiceLineParser<'a> {
    pub fn new(vec: &'a Vec<Token>) -> Self {
        Self { vec, position: 0 }
    }
}

impl Snapshot for SpiceLineParser<'_> {
    type State = usize;

    fn snapshot(&self) -> Self::State {
        self.position
    }

    fn restore(&mut self, state: Self::State) {
        self.position = state;
    }
}

impl TokenStream for SpiceLineParser<'_> {
    fn len(&self) -> usize {
        self.vec.len()
    }

    fn position(&self) -> usize {
        self.position
    }

    fn get(&self, index: usize) -> Option<&Token> {
        self.vec.get(index)
    }

    fn consume(&mut self) {
        self.position += 1;
    }
}

pub struct SpiceFileParser<'a> {
    vec: &'a Vec<Vec<Token>>,
}

impl<'a> SpiceFileParser<'a> {
    pub fn new(vec: &'a Vec<Vec<Token>>) -> Self {
        Self { vec }
    }

    pub fn parse(&mut self) -> ParseResult<Program> {
        let mut iter = self.vec.iter().peekable();
        let mut name = None;
        if let Some(vec) = iter.peek() {
            if vec.len() == 1 && !vec[0].raw.starts_with(".") {
                name = Some(Name(vec[0].clone()));
                iter.next();
            }
        }

        let mut instructions = Vec::new();

        for line in iter {
            // analyze start and end of the line
            let st = line[0].column.0;
            let ed = line[line.len() - 1].column.1;

            let mut parser = SpiceLineParser::new(line);
            let token = parser.token().unwrap();
            if token.raw.starts_with(".") {
                instructions.push(Instruction::Command(
                    parser
                        .try_parse()
                        .map_err(|e| e.fallback_position((st, ed)))?,
                ));
            } else {
                instructions.push(Instruction::Component(
                    parser
                        .try_parse()
                        .map_err(|e| e.fallback_position((st, ed)))?,
                ));
            }
            parser.expect_eof()?;
        }

        let mut stack: Vec<Vec<Instruction>> = vec![vec![]];
        let mut subckts: Vec<SubcktCommand> = vec![];

        for inst in instructions {
            match inst {
                Instruction::Command(command) => match command {
                    Command::Subckt(subckt_command) => {
                        subckts.push(subckt_command);
                        stack.push(vec![]);
                    }
                    Command::Ends(ends_command) => {
                        if let Some(mut subckt) = subckts.pop() {
                            if let Some(name) = ends_command.name {
                                if name != subckt.name {
                                    return Err(ParseError {
                                        reason: "Unmatched .ENDS".to_string(),
                                        position: None,
                                    });
                                }
                            }

                            let instructions = stack.pop().unwrap();
                            subckt.instructions = instructions;
                            stack
                                .last_mut()
                                .unwrap()
                                .push(Instruction::Command(Command::Subckt(subckt)));
                        } else {
                            return Err(ParseError {
                                reason: "Unmatched .ENDS".to_string(),
                                position: None, // TODO: maybe information is lost here
                            });
                        }
                    }
                    _ => {
                        stack
                            .last_mut()
                            .unwrap()
                            .push(Instruction::Command(command));
                    }
                },
                _ => stack.last_mut().unwrap().push(inst),
            }
        }

        if stack.len() != 1 {
            return Err(ParseError {
                reason: "Missing .ENDS for .SUBCKT".to_string(),
                position: None,
            });
        }

        let instructions = stack.pop().unwrap();

        Ok(Program { name, instructions })
    }
}

pub struct SpiceFilePartParser<'a> {
    vec: &'a Vec<Vec<Token>>,
}

impl<'a> SpiceFilePartParser<'a> {
    pub fn new(vec: &'a Vec<Vec<Token>>) -> Self {
        Self { vec }
    }

    pub fn parse(&mut self) -> ParseResult<Program> {
        let mut iter = self.vec.iter().peekable();
        let mut name = None;
        if let Some(vec) = iter.peek() {
            if vec.len() == 1 && !vec[0].raw.starts_with(".") {
                name = Some(Name(vec[0].clone()));
                iter.next();
            }
        }

        let mut instructions = Vec::new();

        for line in iter {
            // analyze start and end of the line
            let st = line[0].column.0;
            let ed = line[line.len() - 1].column.1;

            let mut parser = SpiceLineParser::new(line);
            let token = parser.token().unwrap();
            if token.raw.starts_with(".") {
                instructions.push(Instruction::Command(
                    parser
                        .try_parse()
                        .map_err(|e| e.fallback_position((st, ed)))?,
                ));
            } else {
                instructions.push(Instruction::Component(
                    parser
                        .try_parse()
                        .map_err(|e| e.fallback_position((st, ed)))?,
                ));
            }
            parser.expect_eof()?;
        }

        Ok(Program { name, instructions })
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        ast::{
            command::{
                Command, EndCommand, EndsCommand, IcCommand, LibCommand, ModelCommand,
                SubcktCommand, TranCommand, TranCommandParams,
            },
            component::{
                CComponent, CComponentParams, Component, DComponent, EComponent, FComponent,
                LComponent, LComponentParams, QComponent, RComponent, RComponentParams, VComponent,
                XComponent,
            },
            expression::Number,
            gain::{CurrentGain, SimpleCurrentGain, SimpleVoltageGain, VoltageGain},
            transient::{AcSource, DcSource, Pulse, Sin, Transient},
            variable::{OutputVariable, Suffix, VoltageVariable},
            ExplicitNode, Node, Pairs, Param,
        },
        lexer::SpiceLexer,
    };

    use super::*;

    #[test]
    #[ignore]
    // FIXME: 似乎这个不是 PSpice 语法
    fn test_parse_file_collector() {
        let tokens = SpiceLexer::tokenize(include_str!("../../../models/Collector/Collector.lib"));
        let mut parser = SpiceFileParser::new(&tokens);
        assert_eq!(
            parser.parse(),
            Ok(Program {
                name: None,
                instructions: vec![
                    // .subckt Collector SA_HOp SA_HOn TrigenOut TR_HOp TR_HOn PWMp PWMn  VCOp VCOn PEAK_p PEAK_n gnd
                    Instruction::Command(Command::Subckt(SubcktCommand {
                        name: Name(Atom::from("Collector")),
                        pins: vec![
                            Node(Atom::from("SA_HOp")),
                            Node(Atom::from("SA_HOn")),
                            Node(Atom::from("TrigenOut")),
                            Node(Atom::from("TR_HOp")),
                            Node(Atom::from("TR_HOn")),
                            Node(Atom::from("PWMp")),
                            Node(Atom::from("PWMn")),
                            Node(Atom::from("VCOp")),
                            Node(Atom::from("VCOn")),
                            Node(Atom::from("PEAK_p")),
                            Node(Atom::from("PEAK_n")),
                            Node(Atom::from("gnd")),
                        ],
                        optionals: vec![],
                        params: vec![],
                        texts: vec![],
                        instructions: vec![
                            // YSA_HO SA_HO TrigenOut gnd SA_HOp SA_HOn PARAM: FS = 5meg TACQ = 1E-9 DV = 0.05
                            // VCTR CTR gnd DC 3.3
                            Instruction::Component(Component::V(VComponent {
                                name: Name(Atom::from("VCTR")),
                                node1: Node(Atom::from("CTR")),
                                node2: Node(Atom::from("gnd")),
                                dc: Some(DcSource {
                                    dc: true,
                                    value: Number::from(3.3),
                                }),
                                ac: None,
                                transient: None,
                            })),
                            // YTR_HO TR_HO TrigenOut gnd TR_HOp TR_HOn CTR PARAM: VTH = 1
                            // YTRIGEN TRIGEN TrigenOut gnd PARAM: rdu = 1.0e-3 tdel = 0.0001 v0 = 1.0
                            // YPWM PWM TrigenOut gnd PWMp PWMn PARAM: PVMAX = 5.0 PVMIN = 1.0
                            // YVCO VCO TrigenOut gnd VCOp VCOn PARAM: V1 = 3.3 VOFF = 0
                            // YPEAK_D PEAK_D TrigenOut gnd PEAK_p PEAK_n CTR PARAM: VTH = 1 SLR = 10 RSLR = 1
                            // YLEV_D LEV_D PWMout gnd LEV_p LEV_n PARAM: V0 = 1 V1 = 5 VRL = 2.4 VRU = 2.6 TR = 1us TF = 1us
                            // Vpwm PWMout gnd AC 0.0 PULSE ( 0.0 5.0 0.0 0.01E-3 0.01E-3 0.1E-3 0.2E-3 )
                            Instruction::Component(Component::V(VComponent {
                                name: Name(Atom::from("Vpwm")),
                                node1: Node(Atom::from("PWMout")),
                                node2: Node(Atom::from("gnd")),
                                dc: None,
                                ac: Some(AcSource {
                                    magnitude: Number::from(0.0),
                                    phase: None,
                                }),
                                transient: Some(Transient::Pulse(Pulse {
                                    v1: Number::from(0.0),
                                    v2: Number::from(5.0),
                                    td: Some(Number::from(0.0)),
                                    tr: Some(Number::from(0.01e-3)),
                                    tf: Some(Number::from(0.01e-3)),
                                    pw: Some(Number::from(0.1e-3)),
                                    per: Some(Number::from(0.2e-3)),
                                })),
                            })),
                        ],
                    })),
                ]
            })
        );
    }

    #[test]
    fn test_recursive_subckt() {
        let tokens = SpiceLexer::tokenize(include_str!("../../../models/recursive_subckt.cir"));
        let mut parser = SpiceFileParser::new(&tokens);
        assert_eq!(
            parser.parse(),
            Ok(Program {
                name: None,
                instructions: vec![
                    // .SUBCKT one
                    Instruction::Command(Command::Subckt(SubcktCommand {
                        name: Name(Atom::from("one")),
                        pins: vec![],
                        optionals: vec![],
                        params: vec![],
                        texts: vec![],
                        instructions: vec![
                            // R1 1 1 1
                            Instruction::Component(Component::R(RComponent {
                                name: Name(Atom::from("R1")),
                                node1: Node(Atom::from("1")),
                                node2: Node(Atom::from("1")),
                                model: None,
                                value: Number::from(1.0),
                                params: RComponentParams { tc: None },
                            })),
                            // .SUBCKT two
                            Instruction::Command(Command::Subckt(SubcktCommand {
                                name: Name(Atom::from("two")),
                                pins: vec![],
                                optionals: vec![],
                                params: vec![],
                                texts: vec![],
                                instructions: vec![
                                    // R2 1 1 1
                                    Instruction::Component(Component::R(RComponent {
                                        name: Name(Atom::from("R2")),
                                        node1: Node(Atom::from("1")),
                                        node2: Node(Atom::from("1")),
                                        model: None,
                                        value: Number::from(1.0),
                                        params: RComponentParams { tc: None },
                                    })),
                                ]
                            })),
                            // .SUBCKT one
                            Instruction::Command(Command::Subckt(SubcktCommand {
                                name: Name(Atom::from("one")),
                                pins: vec![],
                                optionals: vec![],
                                params: vec![],
                                texts: vec![],
                                instructions: vec![
                                    // R3 1 1 1
                                    Instruction::Component(Component::R(RComponent {
                                        name: Name(Atom::from("R3")),
                                        node1: Node(Atom::from("1")),
                                        node2: Node(Atom::from("1")),
                                        model: None,
                                        value: Number::from(1.0),
                                        params: RComponentParams { tc: None },
                                    })),
                                ]
                            })),
                            // R4 1 1 1
                            Instruction::Component(Component::R(RComponent {
                                name: Name(Atom::from("R4")),
                                node1: Node(Atom::from("1")),
                                node2: Node(Atom::from("1")),
                                model: None,
                                value: Number::from(1.0),
                                params: RComponentParams { tc: None },
                            })),
                        ]
                    }))
                ]
            })
        );
    }

    #[test]
    fn test_parse_file_rcl() {
        let tokens = SpiceLexer::tokenize(include_str!("../../../models/rcl/RCL.cir"));
        let mut parser = SpiceFileParser::new(&tokens);
        assert_eq!(
            parser.parse(),
            Ok(Program {
                // RCL
                name: Some(Name(Atom::from("RCL"))),
                instructions: vec![
                    // R1 N2 N3 10.0
                    Instruction::Component(Component::R(RComponent {
                        name: Name(Atom::from("R1")),
                        node1: Node(Atom::from("N2")),
                        node2: Node(Atom::from("N3")),
                        model: None,
                        value: Number::from(10.0),
                        params: RComponentParams { tc: None },
                    })),
                    // L1 N2 N1 10.0E-3
                    Instruction::Component(Component::L(LComponent {
                        name: Name(Atom::from("L1")),
                        node1: Node(Atom::from("N2")),
                        node2: Node(Atom::from("N1")),
                        model: None,
                        value: Number::from(10.0e-3),
                        params: LComponentParams { ic: None },
                    })),
                    // L2 N3 N4 10.0E-3
                    Instruction::Component(Component::L(LComponent {
                        name: Name(Atom::from("L2")),
                        node1: Node(Atom::from("N3")),
                        node2: Node(Atom::from("N4")),
                        model: None,
                        value: Number::from(10.0e-3),
                        params: LComponentParams { ic: None },
                    })),
                    // V1I95 N1 0 AC 0.0 SIN ( 0.0 5.0 1.0E3 0.0 0.0 0.0 )
                    Instruction::Component(Component::V(VComponent {
                        name: Name(Atom::from("V1I95")),
                        node1: Node(Atom::from("N1")),
                        node2: Node(Atom::from("0")),
                        dc: None,
                        ac: Some(AcSource {
                            magnitude: Number::from(0.0),
                            phase: None
                        }),
                        transient: Some(Transient::Sin(Sin {
                            v0: Number::from(0.0),
                            vampl: Number::from(5.0),
                            freq: Some(Number::from(1.0e3)),
                            td: Some(Number::from(0.0)),
                            alpha: Some(Number::from(0.0)),
                            theta: Some(Number::from(0.0))
                        })),
                    })),
                    // R2 0 N4 10.0
                    Instruction::Component(Component::R(RComponent {
                        name: Name(Atom::from("R2")),
                        node1: Node(Atom::from("0")),
                        node2: Node(Atom::from("N4")),
                        model: None,
                        value: Number::from(10.0),
                        params: RComponentParams { tc: None },
                    })),
                    // C1 N2 N4 10.0E-6
                    Instruction::Component(Component::C(CComponent {
                        name: Name(Atom::from("C1")),
                        node1: Node(Atom::from("N2")),
                        node2: Node(Atom::from("N4")),
                        model: None,
                        value: Number::from(10.0e-6),
                        params: CComponentParams { ic: None },
                    })),
                    // .TRAN 0.1M 10M
                    Instruction::Command(Command::Tran(TranCommand {
                        op: false,
                        tstep: Number::from(1.0e-4),
                        tstop: Number::from(1.0e-2),
                        tstart: None,
                        tmax: None,
                        params: TranCommandParams {
                            uic: false,
                            skipbp: false,
                        }
                    })),
                    // .LIB
                    Instruction::Command(Command::Lib(LibCommand { filename: None })),
                    // .IC V(2)=3 V(2)=4
                    Instruction::Command(Command::Ic(IcCommand {
                        ics: vec![
                            Pairs {
                                name: OutputVariable::Voltage(VoltageVariable::Node {
                                    node: ExplicitNode(Node(Atom::from("2"))),
                                    suffix: Suffix::None,
                                }),
                                value: Number::from(3.0),
                            },
                            Pairs {
                                name: OutputVariable::Voltage(VoltageVariable::Node {
                                    node: ExplicitNode(Node(Atom::from("2"))),
                                    suffix: Suffix::None,
                                }),
                                value: Number::from(4.0),
                            }
                        ]
                    })),
                    // .END
                    Instruction::Command(Command::End(EndCommand {})),
                ]
            })
        );
    }

    #[test]
    #[ignore]
    /// ignored due to 浮点数误差
    fn test_parse_file_cw7800() {
        let tokens = SpiceLexer::tokenize(include_str!("../../../models/cw7800/cw7800.cir"));
        let mut parser = SpiceFileParser::new(&tokens);
        assert_eq!(
            parser.parse(),
            Ok(Program {
                // cw7800
                name: Some(Name(Atom::from("cw7800"))),
                instructions: vec![
                    // .SUBCKT LM7805C 19 8 21
                    Instruction::Command(Command::Subckt(SubcktCommand {
                        name: Name(Atom::from("LM7805C")),
                        pins: vec![
                            Node(Atom::from("19")),
                            Node(Atom::from("8")),
                            Node(Atom::from("21")),
                        ],
                        optionals: vec![],
                        params: vec![],
                        texts: vec![],
                        instructions: vec![
                            // QAP 4 3 19 QPMOD
                            Instruction::Component(Component::Q(QComponent {
                                name: Name(Atom::from("QAP")),
                                collector: Node(Atom::from("4")),
                                base: Node(Atom::from("3")),
                                emitter: Node(Atom::from("19")),
                                substrate: None,
                                model: Name(Atom::from("QPMOD")),
                                area: None
                            })),
                            // Q1N 19 9 5 QMOD
                            Instruction::Component(Component::Q(QComponent {
                                name: Name(Atom::from("Q1N")),
                                collector: Node(Atom::from("19")),
                                base: Node(Atom::from("9")),
                                emitter: Node(Atom::from("5")),
                                substrate: None,
                                model: Name(Atom::from("QMOD")),
                                area: None
                            })),
                            // Q2N 4 7 5 QMOD OFF
                            Instruction::Component(Component::Q(QComponent {
                                name: Name(Atom::from("Q2N")),
                                collector: Node(Atom::from("4")),
                                base: Node(Atom::from("7")),
                                emitter: Node(Atom::from("5")),
                                substrate: None,
                                model: Name(Atom::from("QMOD")),
                                area: None
                            })),
                            // QSC 4 12 8 QMOD OFF
                            // FIXME: 这个 off 是什么？？？
                            Instruction::Component(Component::Q(QComponent {
                                name: Name(Atom::from("QSC")),
                                collector: Node(Atom::from("4")),
                                base: Node(Atom::from("12")),
                                emitter: Node(Atom::from("8")),
                                substrate: None,
                                model: Name(Atom::from("QMOD")),
                                area: None
                            })),
                            // QOUT 19 1 11 QOUT 10
                            Instruction::Component(Component::Q(QComponent {
                                name: Name(Atom::from("QOUT")),
                                collector: Node(Atom::from("19")),
                                base: Node(Atom::from("1")),
                                emitter: Node(Atom::from("11")),
                                substrate: None,
                                model: Name(Atom::from("QOUT")),
                                area: Some(Number::from(10.0))
                            })),
                            // FEE 5 21 VCHAIN 6
                            Instruction::Component(Component::F(FComponent {
                                name: Name(Atom::from("FEE")),
                                node1: Node(Atom::from("5")),
                                node2: Node(Atom::from("21")),
                                gain: CurrentGain::Simple(SimpleCurrentGain {
                                    vdevice: Name(Atom::from("VCHAIN")),
                                    gain: Number::from(6.0)
                                })
                            })),
                            // EREF 6 21 15 21 5
                            Instruction::Component(Component::E(EComponent {
                                name: Name(Atom::from("EREF")),
                                node1: Node(Atom::from("6")),
                                node2: Node(Atom::from("21")),
                                gain: VoltageGain::Simple(SimpleVoltageGain {
                                    node1: Node(Atom::from("15")),
                                    node2: Node(Atom::from("21")),
                                    gain: Number::from(5.0)
                                })
                            })),
                            // FX 21 15 VCHAIN 1
                            Instruction::Component(Component::F(FComponent {
                                name: Name(Atom::from("FX")),
                                node1: Node(Atom::from("21")),
                                node2: Node(Atom::from("15")),
                                gain: CurrentGain::Simple(SimpleCurrentGain {
                                    vdevice: Name(Atom::from("VCHAIN")),
                                    gain: Number::from(1.0)
                                })
                            })),
                            // VCHAIN 16 2 0
                            Instruction::Component(Component::V(VComponent {
                                name: Name(Atom::from("VCHAIN")),
                                node1: Node(Atom::from("16")),
                                node2: Node(Atom::from("2")),
                                dc: Some(DcSource {
                                    dc: false,
                                    value: Number::from(0.0)
                                }),
                                ac: None,
                                transient: None
                            })),
                            // FAP 3 19 VCHAIN 300M
                            Instruction::Component(Component::F(FComponent {
                                name: Name(Atom::from("FAP")),
                                node1: Node(Atom::from("3")),
                                node2: Node(Atom::from("19")),
                                gain: CurrentGain::Simple(SimpleCurrentGain {
                                    vdevice: Name(Atom::from("VCHAIN")),
                                    gain: Number::from(300.0e6)
                                })
                            })),
                            // JON 2 21 21 JMOD
                            Instruction::Component(Component::Q(QComponent {
                                name: Name(Atom::from("JON")),
                                collector: Node(Atom::from("2")),
                                base: Node(Atom::from("21")),
                                emitter: Node(Atom::from("21")),
                                substrate: None,
                                model: Name(Atom::from("JMOD")),
                                area: None
                            })),
                            // JST 19 21 21 JSTMOD
                            Instruction::Component(Component::Q(QComponent {
                                name: Name(Atom::from("JST")),
                                collector: Node(Atom::from("19")),
                                base: Node(Atom::from("21")),
                                emitter: Node(Atom::from("21")),
                                substrate: None,
                                model: Name(Atom::from("JSTMOD")),
                                area: None
                            })),
                            // DBLK 21 19 DBLK OFF
                            // FIXME: 这个 off 是什么？？？
                            Instruction::Component(Component::D(DComponent {
                                name: Name(Atom::from("DBLK")),
                                node1: Node(Atom::from("21")),
                                node2: Node(Atom::from("19")),
                                model: Name(Atom::from("DBLK")),
                                area: None
                            })),
                            // DXX 19 16 DMOD
                            Instruction::Component(Component::D(DComponent {
                                name: Name(Atom::from("DXX")),
                                node1: Node(Atom::from("19")),
                                node2: Node(Atom::from("16")),
                                model: Name(Atom::from("DMOD")),
                                area: None
                            })),
                            // DREF 13 9 DMOD
                            Instruction::Component(Component::D(DComponent {
                                name: Name(Atom::from("DREF")),
                                node1: Node(Atom::from("13")),
                                node2: Node(Atom::from("9")),
                                model: Name(Atom::from("DMOD")),
                                area: None
                            })),
                            // DAP 3 19 DMOD OFF
                            Instruction::Component(Component::D(DComponent {
                                name: Name(Atom::from("DAP")),
                                node1: Node(Atom::from("3")),
                                node2: Node(Atom::from("19")),
                                model: Name(Atom::from("DMOD")),
                                area: None
                            })),
                            // DSC 12 14 DSCMOD OFF
                            Instruction::Component(Component::D(DComponent {
                                name: Name(Atom::from("DSC")),
                                node1: Node(Atom::from("12")),
                                node2: Node(Atom::from("14")),
                                model: Name(Atom::from("DSCMOD")),
                                area: None
                            })),
                            // DOUT 4 1 DMOD
                            Instruction::Component(Component::D(DComponent {
                                name: Name(Atom::from("DOUT")),
                                node1: Node(Atom::from("4")),
                                node2: Node(Atom::from("1")),
                                model: Name(Atom::from("DMOD")),
                                area: None
                            })),
                            // RBLK 21 19 50K
                            Instruction::Component(Component::R(RComponent {
                                name: Name(Atom::from("RBLK")),
                                node1: Node(Atom::from("21")),
                                node2: Node(Atom::from("19")),
                                model: None,
                                value: Number::from(50.0e3),
                                params: RComponentParams { tc: None },
                            })),
                            // RX 21 15 10K TC=0,-1040N
                            Instruction::Component(Component::R(RComponent {
                                name: Name(Atom::from("RX")),
                                node1: Node(Atom::from("21")),
                                node2: Node(Atom::from("15")),
                                model: None,
                                value: Number::from(10.0e3),
                                params: RComponentParams {
                                    tc: Some((Number::from(0.0), Some(Number::from(-1040.0e-9))))
                                },
                            })),
                            // RSS 6 9 20
                            Instruction::Component(Component::R(RComponent {
                                name: Name(Atom::from("RSS")),
                                node1: Node(Atom::from("6")),
                                node2: Node(Atom::from("9")),
                                model: None,
                                value: Number::from(20.0),
                                params: RComponentParams { tc: None },
                            })),
                            // RDD 13 19 1MEG
                            Instruction::Component(Component::R(RComponent {
                                name: Name(Atom::from("RDD")),
                                node1: Node(Atom::from("13")),
                                node2: Node(Atom::from("19")),
                                model: None,
                                value: Number::from(1.0e6),
                                params: RComponentParams { tc: None },
                            })),
                            // ROMP 7 10 890
                            Instruction::Component(Component::R(RComponent {
                                name: Name(Atom::from("ROMP")),
                                node1: Node(Atom::from("7")),
                                node2: Node(Atom::from("10")),
                                model: None,
                                value: Number::from(890.0),
                                params: RComponentParams { tc: None },
                            })),
                            // RZZ 8 7 5K
                            Instruction::Component(Component::R(RComponent {
                                name: Name(Atom::from("RZZ")),
                                node1: Node(Atom::from("8")),
                                node2: Node(Atom::from("7")),
                                model: None,
                                value: Number::from(5.0e3),
                                params: RComponentParams { tc: None },
                            })),
                            // RSC2 14 19 40K
                            Instruction::Component(Component::R(RComponent {
                                name: Name(Atom::from("RSC2")),
                                node1: Node(Atom::from("14")),
                                node2: Node(Atom::from("19")),
                                model: None,
                                value: Number::from(40.0e3),
                                params: RComponentParams { tc: None },
                            })),
                            // R2 21 8 5K
                            Instruction::Component(Component::R(RComponent {
                                name: Name(Atom::from("R2")),
                                node1: Node(Atom::from("21")),
                                node2: Node(Atom::from("8")),
                                model: None,
                                value: Number::from(5.0e3),
                                params: RComponentParams { tc: None },
                            })),
                            // RAP 4 19 2MEG
                            Instruction::Component(Component::R(RComponent {
                                name: Name(Atom::from("RAP")),
                                node1: Node(Atom::from("4")),
                                node2: Node(Atom::from("19")),
                                model: None,
                                value: Number::from(2.0e6),
                                params: RComponentParams { tc: None },
                            })),
                            // RXX 12 11 530
                            Instruction::Component(Component::R(RComponent {
                                name: Name(Atom::from("RXX")),
                                node1: Node(Atom::from("12")),
                                node2: Node(Atom::from("11")),
                                model: None,
                                value: Number::from(530.0),
                                params: RComponentParams { tc: None },
                            })),
                            // RM 11 1 200K
                            Instruction::Component(Component::R(RComponent {
                                name: Name(Atom::from("RM")),
                                node1: Node(Atom::from("11")),
                                node2: Node(Atom::from("1")),
                                model: None,
                                value: Number::from(200.0e3),
                                params: RComponentParams { tc: None },
                            })),
                            // RSC 8 11 0.27
                            Instruction::Component(Component::R(RComponent {
                                name: Name(Atom::from("RSC")),
                                node1: Node(Atom::from("8")),
                                node2: Node(Atom::from("11")),
                                model: None,
                                value: Number::from(0.27),
                                params: RComponentParams { tc: None },
                            })),
                            // CC2 7 21 0.7N
                            Instruction::Component(Component::C(CComponent {
                                name: Name(Atom::from("CC2")),
                                node1: Node(Atom::from("7")),
                                node2: Node(Atom::from("21")),
                                model: None,
                                value: Number::from(0.7e-9),
                                params: CComponentParams { ic: None },
                            })),
                            // C_OMP 4 10 0.35N
                            Instruction::Component(Component::C(CComponent {
                                name: Name(Atom::from("C_OMP")),
                                node1: Node(Atom::from("4")),
                                node2: Node(Atom::from("10")),
                                model: None,
                                value: Number::from(0.35e-9),
                                params: CComponentParams { ic: None },
                            })),
                            // .MODEL QMOD NPN IS=10F
                            Instruction::Command(Command::Model(ModelCommand {
                                name: Name(Atom::from("QMOD")),
                                reference: None,
                                ty: Name(Atom::from("NPN")),
                                params: vec![Param {
                                    name: Name(Atom::from("IS")),
                                    value: Number::from(10.0e-15),
                                }],
                            })),
                            // .MODEL QSCMOD NPN IS=10F NF=1.1 NR=1.1
                            Instruction::Command(Command::Model(ModelCommand {
                                name: Name(Atom::from("QSCMOD")),
                                reference: None,
                                ty: Name(Atom::from("NPN")),
                                params: vec![
                                    Param {
                                        name: Name(Atom::from("IS")),
                                        value: Number::from(10.0e-15),
                                    },
                                    Param {
                                        name: Name(Atom::from("NF")),
                                        value: Number::from(1.1),
                                    },
                                    Param {
                                        name: Name(Atom::from("NR")),
                                        value: Number::from(1.1),
                                    },
                                ],
                            })),
                            // .MODEL JMOD NJF VTO=-4 BETA=6.25U
                            Instruction::Command(Command::Model(ModelCommand {
                                name: Name(Atom::from("JMOD")),
                                reference: None,
                                ty: Name(Atom::from("NJF")),
                                params: vec![
                                    Param {
                                        name: Name(Atom::from("VTO")),
                                        value: Number::from(-4.0),
                                    },
                                    Param {
                                        name: Name(Atom::from("BETA")),
                                        value: Number::from(6.25e-6),
                                    },
                                ],
                            })),
                            // .MODEL JSTMOD NJF VTO=-4 BETA=147.8125U
                            Instruction::Command(Command::Model(ModelCommand {
                                name: Name(Atom::from("JSTMOD")),
                                reference: None,
                                ty: Name(Atom::from("NJF")),
                                params: vec![
                                    Param {
                                        name: Name(Atom::from("VTO")),
                                        value: Number::from(-4.0),
                                    },
                                    Param {
                                        name: Name(Atom::from("BETA")),
                                        value: Number::from(147.8125e-6),
                                    },
                                ],
                            })),
                            // .MODEL DMOD D
                            Instruction::Command(Command::Model(ModelCommand {
                                name: Name(Atom::from("DMOD")),
                                reference: None,
                                ty: Name(Atom::from("D")),
                                params: vec![],
                            })),
                            // .MODEL QPMOD PNP IS=10F BF=10
                            Instruction::Command(Command::Model(ModelCommand {
                                name: Name(Atom::from("QPMOD")),
                                reference: None,
                                ty: Name(Atom::from("PNP")),
                                params: vec![
                                    Param {
                                        name: Name(Atom::from("IS")),
                                        value: Number::from(10.0e-15),
                                    },
                                    Param {
                                        name: Name(Atom::from("BF")),
                                        value: Number::from(10.0),
                                    },
                                ],
                            })),
                            // .MODEL QOUT NPN IS=10F BF=10K RE=0.1
                            Instruction::Command(Command::Model(ModelCommand {
                                name: Name(Atom::from("QOUT")),
                                reference: None,
                                ty: Name(Atom::from("NPN")),
                                params: vec![
                                    Param {
                                        name: Name(Atom::from("IS")),
                                        value: Number::from(10.0e-15),
                                    },
                                    Param {
                                        name: Name(Atom::from("BF")),
                                        value: Number::from(10.0e3),
                                    },
                                    Param {
                                        name: Name(Atom::from("RE")),
                                        value: Number::from(0.1),
                                    },
                                ],
                            })),
                            // .MODEL DBLK D BV=50
                            Instruction::Command(Command::Model(ModelCommand {
                                name: Name(Atom::from("DBLK")),
                                reference: None,
                                ty: Name(Atom::from("D")),
                                params: vec![Param {
                                    name: Name(Atom::from("BV")),
                                    value: Number::from(50.0),
                                }],
                            })),
                            // .MODEL DSCMOD D BV=7
                            Instruction::Command(Command::Model(ModelCommand {
                                name: Name(Atom::from("DSCMOD")),
                                reference: None,
                                ty: Name(Atom::from("D")),
                                params: vec![Param {
                                    name: Name(Atom::from("BV")),
                                    value: Number::from(7.0),
                                }],
                            })),
                        ]
                    })),
                    // Rout OUT 0 1.0E3
                    Instruction::Component(Component::R(RComponent {
                        name: Name(Atom::from("Rout")),
                        node1: Node(Atom::from("OUT")),
                        node2: Node(Atom::from("0")),
                        model: None,
                        value: Number::from(1.0e3),
                        params: RComponentParams { tc: None },
                    })),
                    // X1I6 IN OUT 0 LM7805C
                    Instruction::Component(Component::X(XComponent {
                        name: Name(Atom::from("X1I6")),
                        pins: vec![
                            Node(Atom::from("IN")),
                            Node(Atom::from("OUT")),
                            Node(Atom::from("0")),
                        ],
                        sname: Name(Atom::from("LM7805C")),
                        params: vec![],
                        texts: vec![],
                    })),
                    // V1I41 IN 0 DC 10.0 AC 0.0
                    Instruction::Component(Component::V(VComponent {
                        name: Name(Atom::from("V1I41")),
                        node1: Node(Atom::from("IN")),
                        node2: Node(Atom::from("0")),
                        dc: Some(DcSource {
                            dc: true,
                            value: Number::from(10.0),
                        }),
                        ac: Some(AcSource {
                            magnitude: Number::from(0.0),
                            phase: None,
                        }),
                        transient: None,
                    })),
                    // .TRAN 0.1M 10M
                    Instruction::Command(Command::Tran(TranCommand {
                        op: false,
                        tstep: Number::from(1.0e-4),
                        tstop: Number::from(1.0e-2),
                        tstart: None,
                        tmax: None,
                        params: TranCommandParams {
                            uic: false,
                            skipbp: false,
                        },
                    })),
                    // .END
                    Instruction::Command(Command::End(EndCommand {})),
                ]
            })
        );
    }
}

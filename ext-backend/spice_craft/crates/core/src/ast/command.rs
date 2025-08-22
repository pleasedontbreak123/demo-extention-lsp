use serde::Serialize;
use spice_proc_macro::{Params, PartialParse, TryParse};

use crate::{ast::Instruction, parse::{
    Element, ElementInfo, ParseError, ParseResult, PartialParse, TokenStream, TryParse,
}};

use super::{
    expression::Text, sweep::Sweep, AcType, Name, Node, Number, Options, OutputVariable, Pairs,
    Param, PrintVariable,
};

/// 交流扫描
///
/// - `.AC <sweep type> <points value>`
/// - `+ <start frequency value> <end frequency value>`
///
/// e.g.
///
/// - `.AC LIN 101 100Hz 200Hz`
/// - `.AC OCT 10 1kHz 16kHz`
/// - `.AC DEC 20 1MEG 100MEG`
#[derive(Debug, Clone, PartialEq, TryParse, Serialize, PartialParse)]
#[grammar(".AC", ty, np, fstart, fend)]
pub struct AcCommand {
    /// sweep type
    pub ty: AcType,
    /// points value
    pub np: Number,
    /// start frequency value
    pub fstart: Number,
    /// end frequency value
    pub fend: Number,
}

/// 直流扫描
///
/// - `.DC <sweep1> <sweep2> ...`
///
/// e.g.
///
/// - `.DC VIN -.25 .25 .05`
/// - `.DC LIN I2 5mA -2mA 0.1mA`
/// - `.DC VCE 0V 10V .5V IB 0mA 1mA 50uA`
/// - `.DC RES RMOD(R) 0.9 1.1 .001`
///
/// - `.DC VIN -5V 10V 0.25V`
/// - `.DC LIN IIN 50MA -50MA 1MA`
/// - `.DC VA 0 15V 0.5V IA 0 1MA 0.05MA`
/// - `.DC RES RMOD(R) 0.9 1.1 0.001`
/// - `.DC DEC NPN QM(IS) 1E-18 1E-14 10`
/// - `.DC TEMP LIST 0 50 80 100 150`
/// - `.DC PARAM Vsupply -15V 15V 0.5V`
#[derive(Debug, Clone, PartialEq, TryParse, Serialize)]
#[grammar(".DC", sweeps)]
pub struct DcCommand {
    pub sweeps: Vec<Sweep>,
}

/// 结束定义
///
/// - `.END`
#[derive(Debug, Clone, PartialEq, TryParse, Serialize)]
#[grammar(".END")]
pub struct EndCommand {}

/// 傅里叶分析
///
/// - `.FOUR <freq> [<np>] <var1> <var2> ...`
///
/// e.g.
///
/// - `.FOUR 10kHz V(5) V(6,7) I(VSENS3)`
/// - `.FOUR 60Hz 20 V(17)`
/// - `.FOUR 10kHz V([OUT1],[OUT2])`
#[derive(Debug, Clone, PartialEq, TryParse, Serialize, PartialParse)]
#[grammar(".FOUR", freq, np, variables)]
pub struct FourCommand {
    pub freq: Number,
    pub np: Option<usize>,
    pub variables: Vec<OutputVariable>,
}

/// 函数定义
///
/// TODO: need more specifications
///
/// - `.FUNC <name> ( <arg>* ) { <body> }`
///
/// e.g.
///
/// - `.FUNC E(x) {exp(x)}`
/// - `.FUNC DECAY(CNST) {E(-CNST*TIME)}`
/// - `.FUNC TRIWAV(x) {ACOS(COS(x))/3.14159}`
/// - `.FUNC MIN3(A,B,C) {MIN(A,MIN(B,C))}`
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct FuncCommand {}

/// 全局
///
/// 定义一个全局节点，然后在任何地方引用都表示该节点
///
/// - `.GLOBAL <node>`
#[derive(Debug, Clone, PartialEq, Serialize, TryParse)]
#[grammar(".GLOBAL", node)]
pub struct GlobalCommand {
    pub node: Node,
}

/// 瞬态初始状态
///
/// - `.IC <ic1> <ic2> ...`
///
/// e.g.
///
/// - `.IC V(1)=2.5 V(5)=1.7 V(7)=0.5`
/// - `.IC V(2)=3.4 V(102)=0 V(3)=-1V I(L1)=2uAmp`
/// - `.IC V(InPlus,InMinus)=1e-3 V(100,133)=5.0V`
#[derive(Debug, Clone, PartialEq, Serialize, TryParse)]
#[grammar(".IC", ics)]
pub struct IcCommand {
    pub ics: Vec<Pairs<OutputVariable, Number>>,
}

/// 引入文件
///
/// - `.INC <file name>`
///
/// e.g.
///
/// - `.INC "SETUP.CIR"`
/// - `.INC "C:\LIB\VCO.CIR"`
#[derive(Debug, Clone, PartialEq, Serialize, TryParse)]
#[grammar(".INC", filename)]
pub struct IncCommand {
    pub filename: Text,
}

/// 引入库
///
/// - `.LIB [<file name>]`
///
/// e.g.
///
/// - `.LIB`
/// - `.LIB linear.lib`
/// - `.LIB "C:\lib\bipolar.lib"`
#[derive(Debug, Clone, PartialEq, Serialize, TryParse)]
#[grammar(".LIB", filename)]
pub struct LibCommand {
    pub filename: Option<Text>,
}

// 不实现 LOADBIAS 相关功能
//
// /// 引入 bios 文件
// ///
// /// - `.LOADBIAS <file name>`
// ///
// /// e.g.
// ///
// /// - `.LOADBIAS "SAVETRAN.NOD"`
// /// - `.LOADBIAS "C:\PROJECT\INIT.FIL"`
// #[derive(Debug, Clone, PartialEq, Serialize)]
// pub struct LoadBiasCommand {
//     pub filename: Atom,
// }

/// 蒙特卡罗分析
///
/// TODO: 懒得写
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct McCommand {}

/// 模型
///
/// - `.MODEL <model name> [AKO: <reference model name>]`
/// - `+ <model type>`
/// - `+ ( [<parameter name> = <value> [tolerance specification]]*`
/// - `+   [T_MEASURED=<value>] [ [T_ABS=<value>] or`
/// - `+   [T_REL_GLOBAL=<value>] or [T_REL_LOCAL=<value>] ] )`
///
/// - *括号可选*
///
/// e.g.
///
/// - `.MODEL RMAX RES (R=1.5 TC1=.02 TC2=.005)`
/// - `.MODEL DNOM D (IS=1E-9)`
/// - `.MODEL QDRIV NPN (IS=1E-7 BF=30)`
/// - `.MODEL MLOAD NMOS(LEVEL=1 VTO=.7 CJ=.02pF)`
/// - `.MODEL CMOD CAP (C=1 DEV 5%)`
/// - `.MODEL DLOAD D (IS=1E-9 DEV .5% LOT 10%)`
/// - `.MODEL RTRACK RES (R=1 DEV/GAUSS 1% LOT/UNIFORM 5%)`
/// - `.MODEL QDR2 AKO:QDRIV NPN (BF=50 IKF=50m)`
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ModelCommand {
    pub name: Name,
    pub reference: Option<Name>,
    pub ty: Name,
    /// TODO: 加入 tolerance （语法？）
    pub params: Vec<Param<Number>>,
}

impl<T> TryParse<ModelCommand> for T
where
    T: TokenStream,
{
    fn try_parse(&mut self) -> ParseResult<ModelCommand> {
        self.expect(".MODEL")?;
        let name = self.try_parse()?;
        let reference = if self.matches_consume("AKO") {
            self.expect(":")?;
            Some(self.try_parse()?)
        } else {
            None
        };
        let ty = self.try_parse()?;
        let has_brackets = self.matches_consume("(");
        let params = self.try_parse()?;
        if has_brackets {
            self.expect(")")?;
        }

        Ok(ModelCommand {
            name,
            reference,
            ty,
            params,
        })
    }
}

/// 设置节点
///
/// - `.NODESET <V(<node>[,<node>])=<value>>*`
/// - `.NODESET <I(<inductor>)=<value>>`
///
/// e.g.
///
/// - `.NODESET V(1)=2V V(2)=3V`
/// - `.NODESET V(2)=3.4 V(102)=0 V(3)=-1V I(L1)=2uAmp`
/// - `.NODESET V(InPlus,InMinus)=1e-3 V(100,133)=5.0V`
#[derive(Debug, Clone, PartialEq, Serialize, TryParse)]
#[grammar(".NODESET", nodesets)]
pub struct NodeSetCommand {
    pub nodesets: Vec<Pairs<OutputVariable, Number>>,
}

/// 噪声分析
///
/// - `.NOISE <variable> <source> [<m>]`
///
/// e.g.
///
/// - `.NOISE V(4,5) VIN`
/// - `.NOISE V(6) IIN`
/// - `.NOISE V(10) V1 10`
#[derive(Debug, Clone, PartialEq, Serialize, TryParse)]
#[grammar(".NOISE", variable, source, m)]
pub struct NoiseCommand {
    pub variable: OutputVariable,
    pub source: Name,
    pub m: Option<usize>,
}

/// 启动原神
///
/// - `.OP`
#[derive(Debug, Clone, PartialEq, Serialize, TryParse)]
#[grammar(".OP")]
pub struct OpCommand {}

/// 分析选项
///
/// - `.OPTIONS <option1> <option2> ...`
///
/// e.g.
///
/// - `.OPTIONS NOECHO NOMOD DEFL=12u DEFW=8u DEFAD=150p DEFAS=150p`
/// - `.OPTIONS ACCT RELTOL=.01`
/// - `.OPTIONS DISTRIBUTION=GAUSS`
/// - `.OPTIONS DISTRIBUTION=USERDEF1`
#[derive(Debug, Clone, PartialEq, Serialize, TryParse)]
#[grammar(".OPTIONS", options)]
pub struct OptionsCommand {
    pub options: Vec<Options<Number>>,
}

/// 参数定义
///
/// `.PARAM <param1> <param2> ...`
///
/// e.g.
///
/// - `.PARAM VSUPPLY = 5V`
/// - `.PARAM VCC = 12V, VEE = -12V`
/// - `.PARAM BANDWIDTH = {100kHz/3}`
/// - `.PARAM PI = 3.14159, TWO_PI = {2*3.14159}`
/// - `.PARAM VNUM = {2*TWO_PI}`
#[derive(Debug, Clone, PartialEq, Serialize, TryParse)]
#[grammar(".PARAM", params)]
pub struct ParamCommand {
    pub params: Vec<Param<Number>>,
}

/// 能够输出的东西来源
#[derive(Debug, Clone, PartialEq, Serialize, TryParse)]
#[serde(tag = "type")]
pub enum ProbeSource {
    /// 瞬态响应
    /// - `TRAN`
    #[matches("TRAN")]
    Tran,
    /// 直流扫描
    /// - `DC`
    #[matches("DC")]
    Dc,
    /// 频率响应
    /// - `AC`
    #[matches("AC")]
    Ac,
    /// 噪声分析
    /// - `NOISE`
    #[matches("NOISE")]
    Noise,
}

/// 绘图输出
///
/// - `.PLOT <source> <var1> <var2> ...`
///
/// e.g.
///
/// - `.PLOT DC V(3) V(2,3) V(R1) I(VIN) I(R2) IB(Q13) VBE(Q13)`
/// - `.PLOT AC VM(2) VP(2) VM(3,4) VG(5) VDB(5) IR(D4)`
/// - `.PLOT NOISE INOISE ONOISE DB(INOISE) DB(ONOISE)`
/// - `.PLOT TRAN V(3) V(2,3) (0,5V) ID(M2) I(VCC) (-50mA,50mA)`
/// - `.PLOT TRAN D(QA) D(QB) V(3) V(2,3)`
/// - `.PLOT TRAN V(3) V(R1) V([RESET])`
#[derive(Debug, Clone, PartialEq, Serialize, TryParse)]
#[grammar(".PLOT", source, variables)]
pub struct PlotCommand {
    pub source: ProbeSource,
    pub variables: Vec<PrintVariable>,
}

/// 列表形式输出
///
/// - `.PRINT[/DGTLCHG] <source> <var1> <var2> ...`
///
/// e.g.
///
/// - `.PRINT DC V(3) V(2,3) V(R1) I(VIN) I(R2) IB(Q13) VBE(Q13)`
/// - `.PRINT AC VM(2) VP(2) VM(3,4) VG(5) VDB(5) IR(6) II(7)`
/// - `.PRINT NOISE INOISE ONOISE DB(INOISE) DB(ONOISE)`
/// - `.PRINT TRAN V(3) V(2,3) ID(M2) I(VCC)`
/// - `.PRINT TRAN D(QA) D(QB) V(3) V(2,3)`
/// - `.PRINT/DGTLCHG TRAN QA QB RESET`
/// - `.PRINT TRAN V(3) V(R1) V([RESET])`
#[derive(Debug, Clone, PartialEq, Serialize, TryParse)]
#[grammar(".PRINT", dgtlchg("/", "DGTLCHG"), source, variables)]
pub struct PrintCommand {
    pub dgtlchg: bool,
    pub source: ProbeSource,
    pub variables: Vec<PrintVariable>,
}

/// 屏幕图形输出指令
///
/// - `.PROBE[/CSDF] <output variable>*`
///
/// e.g.
///
/// - `.PROBE`
/// - `.PROBE V(3) V(2,3) V(R1) I(VIN) I(R2) IB(Q13) VBE(Q13)`
/// - `.PROBE/CSDF`
/// - `.PROBE V(3) V(R1) V([RESET])`
/// - `.PROBE D(QBAR)`
#[derive(Debug, Clone, PartialEq, Serialize, TryParse)]
#[grammar(".PROBE", csdf("/", "CSDF"), variables)]
pub struct ProbeCommand {
    pub csdf: bool,
    pub variables: Vec<PrintVariable>,
}

// 不实现
// /// 保存偏置
// /// TODO: what is this??
// #[derive(Debug, Clone, PartialEq, Serialize)]
// pub struct SaveBiasCommand {}

/// 小信号灵敏度
///
/// - `.SENS <output variable>*`
///
/// e.g.
///
/// - `.SENS V(9) V(4,3) V(17) I(VCC)`
#[derive(Debug, Clone, PartialEq, Serialize, TryParse)]
#[grammar(".SENS", variables)]
pub struct SensCommand {
    pub variables: Vec<OutputVariable>,
}

/// 扫描分析
///
/// - `.STEP <sweep1> <sweep2> ...`
///
/// e.g.
///
/// - `.STEP VCE 0V 10V .5V`
/// - `.STEP LIN I2 5mA -2mA 0.1mA`
/// - `.STEP RES RMOD(R) 0.9 1.1 .001`
/// - `.STEP DEC NPN QFAST(IS) 1E-18 1E-14 5`
/// - `.STEP TEMP LIST 0 20 27 50 80 100`
/// - `.STEP PARAM CenterFreq 9.5kHz 10.5kHz 50Hz`
#[derive(Debug, Clone, PartialEq, Serialize, TryParse)]
#[grammar(".STEP", sweeps)]
pub struct StepCommand {
    pub sweeps: Vec<Sweep>,
}

// /// 这是什么？
// ///
// /// - `.STMLIB <file name>`
// ///
// /// e.g.
// ///
// /// - `.STMLIB mylib.stl`
// /// - `.STMLIB volts.stl`
// /// - `.STMLIB dgpulse`
// #[derive(Debug, Clone, PartialEq, Serialize)]
// pub struct StimLibCommand {
//     pub filename: Atom,
// }

// /// 这是什么？
// ///
// /// TODO: 这是什么？？
// #[derive(Debug, Clone, PartialEq, Serialize)]
// pub struct StimulusCommand {}

/// 子电路定义
///
/// FIXME: 到底是 PARAMS: 还是 PARAM: ？
///
/// - `.SUBCKT <name> [node]*`
/// - `+ [OPTIONAL: < <interface node> = <default value> >*]`
/// - `+ [PARAMS: < <name> = <value> >* ]`
/// - `+ [TEXT: < <name> = <text value> >* ]`
///
/// e.g.
///
/// - `.SUBCKT OPAMP 1 2 101 102 17`
/// - `.SUBCKT FILTER INPUT, OUTPUT PARAMS: CENTER=100kHz, BANDWIDTH=10kHz`
/// - `.SUBCKT PLD IN1 IN2 IN3 OUT1 PARAMS: MNTYMXDLY=0 IO_LEVEL=0 TEXT: JEDEC_FILE="PROG.JED"`
/// - `.SUBCKT 74LS00 A B Y OPTIONAL: DPWR=$G_DPWR DGND=$G_DGND PARAMS: MNTYMXDLY=0 IO_LEVEL=0`
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SubcktCommand {
    pub name: Name,
    pub pins: Vec<Node>,
    pub optionals: Vec<Pairs<Node, Node>>,
    pub params: Vec<Param<Number>>,
    pub texts: Vec<Param<Text>>,
    pub instructions: Vec<Instruction>,
}

impl<T> TryParse<SubcktCommand> for T
where
    T: TokenStream,
{
    fn try_parse(&mut self) -> ParseResult<SubcktCommand> {
        // .SUBCKT <name> <pin1> <pin2> ... [PARAM: <param1> <param2> ...]
        self.expect(".SUBCKT")?;
        let name = self.try_parse()?;

        let mut pins = Vec::new();
        while !self.is_eof()
            && !self.matches("OPTIONAL")
            && !self.matches("PARAM")
            && !self.matches("TEXT")
        {
            let pin = self.try_parse()?;
            pins.push(pin);
        }

        let mut optionals = Vec::new();
        if self.matches_consume("OPTIONAL") {
            self.expect(":")?;
            while !self.is_eof() && !self.matches("PARAM") && !self.matches("TEXT") {
                optionals.push(self.try_parse()?);
            }
        }

        let mut params = Vec::new();
        if self.matches_consume("PARAM") {
            self.expect(":")?;
            while !self.is_eof() && !self.matches("TEXT") {
                params.push(self.try_parse()?);
            }
        }

        let mut texts = Vec::new();
        if self.matches_consume("TEXT") {
            self.expect(":")?;
            while !self.is_eof() {
                texts.push(self.try_parse()?);
            }
        }

        let instructions = Vec::new();

        Ok(SubcktCommand {
            name,
            pins,
            optionals,
            params,
            texts,
            instructions,
        })
    }
}

/// 结束子电路定义
///
/// - `.ENDS [<name>]`
///
/// e.g.
///
/// - `.ENDS`
/// - `.ENDS OPAMP`
#[derive(Debug, Clone, PartialEq, Serialize, TryParse)]
#[grammar(".ENDS", name)]
pub struct EndsCommand {
    pub name: Option<Name>,
}

/// 指定系统工作温度
///
/// - `.TEMP <temperature value>*`
///
/// e.g.
///
/// - `.TEMP 50`
/// - `.TEMP 0 25 50 100`
#[derive(Debug, Clone, PartialEq, Serialize, TryParse)]
#[grammar(".TEMP", temps)]
pub struct TempCommand {
    pub temps: Vec<Number>,
}

/// 文本参数
///
/// - `.TEXT < <name> = "<text value>" >*`
/// - `.TEXT < <name> = | <text expression> | >*`
///
/// e.g.
///
/// - `.TEXT MYFILE = "FILENAME.EXT"`
/// - `.TEXT FILE = "ROM.DAT", FILE2 = "ROM2.DAT"`
/// - `.TEXT PROGDAT = |"ROM"+TEXTINT(RUN_NO)+".DAT"|`
/// - `.TEXT DATA1 = "PLD.JED", PROGDAT = |"\PROG\DAT\"+FILENAME|`
#[derive(Debug, Clone, PartialEq, Serialize, TryParse)]
#[grammar(".TEXT", texts)]
pub struct TextCommand {
    pub texts: Vec<Param<Text>>,
}

/// 王俊凯
///
/// - `.TF <output variable> <input source name>`
///
/// e.g.
///
/// - `.TF V(10) VIN`
/// - `.TF I(VN) IIN`
#[derive(Debug, Clone, PartialEq, Serialize, TryParse)]
#[grammar(".TF", output, input)]
pub struct TfCommand {
    pub output: OutputVariable,
    pub input: Name,
}

/// 瞬态分析，时域响应
///
/// FIXME: 英文与中文版本有冲突
///
/// - `.TRAN[/OP] <print step value> <final time value>`
/// - `+ [no-print value [step ceiling value]] [SKIPBP]`
///
/// e.g.
///
/// - `.TRAN 5US 1MS`
/// - `.TRAN 5US 1MS 200US 0.1NS`
/// - `.TRAN 5US 1MS 200US 0.1NS UIC`
/// - `.TRAN/OP 5US 1MS 200US 0.1NS UIC`
/// - `.TRAN[/OP] <tstep> <tstop> [<tstart>] [<tmax>] [UIC]`
#[derive(Debug, Clone, PartialEq, Serialize, TryParse)]
#[grammar(".TRAN", op("/", "OP"), tstep, tstop, tstart, tmax, params)]
pub struct TranCommand {
    pub op: bool,
    pub tstep: Number,
    pub tstop: Number,
    pub tstart: Option<Number>,
    pub tmax: Option<Number>,
    pub params: TranCommandParams,
}

#[derive(Debug, Clone, PartialEq, Serialize, Params)]
pub struct TranCommandParams {
    #[param(name = "UIC")]
    pub uic: bool,
    #[param(name = "SKIPBP")]
    pub skipbp: bool,
}

// /// 什么东西？
// ///
// /// TODO: 什么东西？？？
// #[derive(Debug, Clone, PartialEq, Serialize)]
// pub struct VectorCommand {}

/// 观察统计，和 PRINT 和 PROBE 差不多
#[derive(Debug, Clone, PartialEq, Serialize, TryParse)]
#[grammar(".WATCH", source, variables)]
pub struct WatchCommand {
    pub source: ProbeSource,
    pub variables: Vec<PrintVariable>,
}

/// 设置宽度
///
/// Note: 只有中文版里面有
///
/// - `.WIDTH OUT=<value>`
#[derive(Debug, Clone, PartialEq, Serialize, TryParse)]
#[grammar(".WIDTH", "OUT", "=", width)]
pub struct WidthCommand {
    pub width: Number,
}

/// 点开头的命令
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "type")]
pub enum Command {
    Ac(AcCommand),
    Dc(DcCommand),
    End(EndCommand),
    Four(FourCommand),
    Func(FuncCommand),
    Global(GlobalCommand),
    Ic(IcCommand),
    Inc(IncCommand),
    Lib(LibCommand),
    Mc(McCommand),
    Model(ModelCommand),
    NodeSet(NodeSetCommand),
    Noise(NoiseCommand),
    Op(OpCommand),
    Options(OptionsCommand),
    Param(ParamCommand),
    Plot(PlotCommand),
    Print(PrintCommand),
    Probe(ProbeCommand),
    Sens(SensCommand),
    Step(StepCommand),
    Subckt(SubcktCommand),
    Ends(EndsCommand),
    Temp(TempCommand),
    Text(TextCommand),
    Tf(TfCommand),
    Tran(TranCommand),
    Watch(WatchCommand),
    Width(WidthCommand),
}

// TODO: 核对 Command 语法
impl<T> TryParse<Command> for T
where
    T: TokenStream,
{
    fn try_parse(&mut self) -> ParseResult<Command> {
        let token = self.token()?;
        match &token.to_uppercase()[..] {
            ".AC" => Ok(Command::Ac(self.try_parse()?)),
            // ".ALIASES" => unimplemented!(),
            // ".ENDALIASES" => unimplemented!(),
            ".DC" => Ok(Command::Dc(self.try_parse()?)),
            // ".DISTRIBUTION" => todo!(),
            ".END" => Ok(Command::End(self.try_parse()?)),
            // ".EXTERNAL" => todo!(),
            ".FOUR" => Ok(Command::Four(self.try_parse()?)),
            ".FUNC" => todo!(),
            ".GLOBAL" => Ok(Command::Global(self.try_parse()?)),
            ".IC" => Ok(Command::Ic(self.try_parse()?)),
            ".INC" => Ok(Command::Inc(self.try_parse()?)),
            ".LIB" => Ok(Command::Lib(self.try_parse()?)),
            // ".LOADBIAS" => unimplemented!(),
            ".MC" => todo!(),
            ".MODEL" => Ok(Command::Model(self.try_parse()?)),
            ".NODESET" => Ok(Command::NodeSet(self.try_parse()?)),
            ".NOISE" => Ok(Command::Noise(self.try_parse()?)),
            ".OP" => Ok(Command::Op(self.try_parse()?)),
            ".OPTIONS" => Ok(Command::Options(self.try_parse()?)),
            ".PARAM" => Ok(Command::Param(self.try_parse()?)),
            ".PLOT" => Ok(Command::Plot(self.try_parse()?)),
            ".PRINT" => Ok(Command::Print(self.try_parse()?)),
            ".PROBE" => Ok(Command::Probe(self.try_parse()?)),
            // ".SAVEBIAS" => todo!(),
            ".SENS" => Ok(Command::Sens(self.try_parse()?)),
            ".STEP" => Ok(Command::Step(self.try_parse()?)),
            // ".STMLIB" => unimplemented!(),
            // ".STIMULUS" => unimplemented!(),
            ".SUBCKT" => Ok(Command::Subckt(self.try_parse()?)),
            ".ENDS" => Ok(Command::Ends(self.try_parse()?)),
            ".TEMP" => Ok(Command::Temp(self.try_parse()?)),
            ".TEXT" => Ok(Command::Text(self.try_parse()?)),
            ".TF" => Ok(Command::Tf(self.try_parse()?)),
            ".TRAN" => Ok(Command::Tran(self.try_parse()?)),
            // ".VECTOR" => todo!(),
            ".WATCH" => Ok(Command::Watch(self.try_parse()?)),
            ".WIDTH" => Ok(Command::Width(self.try_parse()?)),

            name if name.starts_with(".") => Err(ParseError {
                reason: format!("Unknown command `{}'", token.raw),
                position: Some(token.column),
            }),

            _ => Err(ParseError {
                reason: format!("Expect `{}' to be a Command name", token.raw),
                position: Some(token.column),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        ast::{
            sweep::{Lin, List, SweepName},
            variable::{NoiseVariable, Suffix, VoltageVariable},
            Atom, ExplicitNode, Limits,
        },
        spice_test_err, spice_test_ok,
    };

    use super::*;

    #[test]
    fn test_parse_command_ac() {
        spice_test_ok!(
            ".AC LIN 101 100Hz 200Hz",
            AcCommand {
                ty: AcType::Lin,
                np: 101f64.into(),
                fstart: 100f64.into(),
                fend: 200f64.into(),
            }
        );
        spice_test_ok!(
            ".AC OCT 10 1kHz 16kHz",
            AcCommand {
                ty: AcType::Oct,
                np: 10f64.into(),
                fstart: 1000f64.into(),
                fend: 16000f64.into(),
            }
        );
        spice_test_ok!(
            ".AC DEC 20 1MEG 100MEG",
            AcCommand {
                ty: AcType::Dec,
                np: 20f64.into(),
                fstart: 1e6.into(),
                fend: 1e8.into(),
            }
        );
    }

    #[test]
    fn test_error_field() {
        spice_test_err!(".AC HEX 10 40 40", AcCommand);
    }

    #[test]
    fn test_parse_command_dc() {
        spice_test_ok!(
            ".DC V2 LIST 1 2 3",
            DcCommand {
                sweeps: vec![Sweep::List(List {
                    swname: SweepName::Power {
                        name: Name(Atom::from("V2"))
                    },
                    values: vec![Number::from(1.0), Number::from(2.0), Number::from(3.0),]
                })]
            }
        )
    }

    #[test]
    fn test_parse_command_end() {
        spice_test_ok!(".END", EndCommand {});
        spice_test_err!(".ENDX", EndCommand);
    }

    #[test]
    fn test_parse_command_four() {
        spice_test_ok!(
            ".FOUR 60Hz 20 V(17)",
            FourCommand {
                freq: Number::from(60.0),
                np: Some(20),
                variables: vec![OutputVariable::Voltage(VoltageVariable::Node {
                    node: ExplicitNode(Node(Atom::from("17"))),
                    suffix: Suffix::None
                })],
            }
        );
        spice_test_ok!(
            ".FOUR 10kHz V([OUT1],[OUT2])",
            FourCommand {
                freq: Number::from(10e3),
                np: None,
                variables: vec![OutputVariable::Voltage(VoltageVariable::Node2 {
                    node1: ExplicitNode(Node(Atom::from("OUT1"))),
                    node2: ExplicitNode(Node(Atom::from("OUT2"))),
                    suffix: Suffix::None
                })],
            }
        );
    }

    #[test]
    #[ignore]
    // TODO: 完善 command_func 测试
    fn test_parse_command_func() {
        todo!()
    }

    #[test]
    fn test_parse_command_global() {
        spice_test_ok!(
            ".GLOBAL V1",
            GlobalCommand {
                node: Node(Atom::from("V1"))
            }
        );
        spice_test_err!(".GLOBAL", GlobalCommand);
        spice_test_err!(".GLOBALX V2", GlobalCommand);
    }

    #[test]
    fn test_parse_command_ic() {
        spice_test_ok!(
            ".IC V(3)=4",
            IcCommand {
                ics: vec![Pairs {
                    name: OutputVariable::Voltage(VoltageVariable::Node {
                        node: ExplicitNode(Node(Atom::from("3"))),
                        suffix: Suffix::None
                    }),
                    value: Number::from(4.0),
                }]
            }
        )
    }

    #[test]
    fn test_parse_command_inc() {
        spice_test_ok!(
            r#".inc "test.lib""#,
            IncCommand {
                filename: Text(Atom::from("\"test.lib\""))
            }
        );
    }

    #[test]
    fn test_parse_command_lib() {
        spice_test_ok!(
            r#".lib "test.lib""#,
            LibCommand {
                filename: Some(Text(Atom::from("\"test.lib\"")))
            }
        );
        spice_test_ok!(".lib", LibCommand { filename: None });
    }

    #[test]
    #[ignore]
    // TODO: 完善 command_mc 测试
    fn test_parse_command_mc() {
        todo!()
    }

    #[test]
    fn test_parse_command_model() {
        spice_test_ok!(
            ".MODEL QDR2 AKO:QDRIV NPN (BF=50 IKF=50m)",
            ModelCommand {
                name: Name(Atom::from("QDR2")),
                reference: Some(Name(Atom::from("QDRIV"))),
                ty: Name(Atom::from("NPN")),
                params: vec![
                    Param {
                        name: Name(Atom::from("BF")),
                        value: Number::from(50.0),
                    },
                    Param {
                        name: Name(Atom::from("IKF")),
                        value: Number::from(50.0e-3),
                    },
                ]
            }
        )
    }

    #[test]
    fn test_parse_command_nodeset() {
        spice_test_ok!(
            ".NODESET V(10)=10",
            NodeSetCommand {
                nodesets: vec![Pairs {
                    name: OutputVariable::Voltage(VoltageVariable::Node {
                        node: ExplicitNode(Node(Atom::from("10"))),
                        suffix: Suffix::None
                    }),
                    value: Number::from(10.0),
                }]
            }
        );
    }

    #[test]
    fn test_parse_command_noise() {
        spice_test_ok!(
            ".NOISE V(10) V1 10",
            NoiseCommand {
                variable: OutputVariable::Voltage(VoltageVariable::Node {
                    node: ExplicitNode(Node(Atom::from("10"))),
                    suffix: Suffix::None
                }),
                source: Name(Atom::from("V1")),
                m: Some(10),
            }
        );
    }

    #[test]
    fn test_parse_command_op() {
        spice_test_ok!(".OP", OpCommand {});
    }

    #[test]
    fn test_parse_command_options() {
        spice_test_ok!(
            ".OPTIONS A B=2.3 C=2.3 D E=33.3",
            OptionsCommand {
                options: vec![
                    Options {
                        name: Name(Atom::from("A")),
                        value: None,
                    },
                    Options {
                        name: Name(Atom::from("B")),
                        value: Some(Number::from(2.3)),
                    },
                    Options {
                        name: Name(Atom::from("C")),
                        value: Some(Number::from(2.3)),
                    },
                    Options {
                        name: Name(Atom::from("D")),
                        value: None,
                    },
                    Options {
                        name: Name(Atom::from("E")),
                        value: Some(Number::from(33.3)),
                    },
                ]
            }
        );
    }

    #[test]
    fn test_parse_command_param() {
        spice_test_ok!(
            ".PARAM v=22 d=4.3",
            ParamCommand {
                params: vec![
                    Param {
                        name: Name(Atom::from("v")),
                        value: Number::from(22.0),
                    },
                    Param {
                        name: Name(Atom::from("d")),
                        value: Number::from(4.3),
                    }
                ]
            }
        )
    }

    #[test]
    fn test_parse_command_plot() {
        spice_test_ok!(
            ".PLOT DC V(2) V(4) (2, 4)",
            PlotCommand {
                source: ProbeSource::Dc,
                variables: vec![
                    PrintVariable {
                        variable: OutputVariable::Voltage(VoltageVariable::Node {
                            node: ExplicitNode(Node(Atom::from("2"))),
                            suffix: Suffix::None
                        }),
                        limits: None,
                    },
                    PrintVariable {
                        variable: OutputVariable::Voltage(VoltageVariable::Node {
                            node: ExplicitNode(Node(Atom::from("4"))),
                            suffix: Suffix::None
                        }),
                        limits: Some(Limits {
                            lower: Number::from(2.0),
                            upper: Number::from(4.0),
                        }),
                    },
                ]
            }
        )
    }

    #[test]
    fn test_parse_command_print() {
        spice_test_ok!(
            ".PRINT NOISE INOISE ONOISE DB(INOISE) DB(ONOISE)",
            PrintCommand {
                dgtlchg: false,
                source: ProbeSource::Noise,
                variables: vec![
                    PrintVariable {
                        variable: OutputVariable::Noise(NoiseVariable::Inoise),
                        limits: None
                    },
                    PrintVariable {
                        variable: OutputVariable::Noise(NoiseVariable::Onoise),
                        limits: None
                    },
                    PrintVariable {
                        variable: OutputVariable::Noise(NoiseVariable::DbInoise),
                        limits: None
                    },
                    PrintVariable {
                        variable: OutputVariable::Noise(NoiseVariable::DbOnoise),
                        limits: None
                    },
                ],
            }
        )
    }

    #[test]
    fn test_parse_command_probe() {
        spice_test_ok!(
            ".PROBE",
            ProbeCommand {
                csdf: false,
                variables: vec![]
            }
        );
        spice_test_ok!(
            ".PROBE/CSDF",
            ProbeCommand {
                csdf: true,
                variables: vec![]
            }
        );
    }

    #[test]
    fn test_parse_command_sens() {
        spice_test_ok!(
            ".SENS V(2)",
            SensCommand {
                variables: vec![OutputVariable::Voltage(VoltageVariable::Node {
                    node: ExplicitNode(Node(Atom::from("2"))),
                    suffix: Suffix::None
                })],
            }
        )
    }

    #[test]
    fn test_parse_command_step() {
        spice_test_ok!(
            ".STEP PARAM CenterFreq 9.5kHz 10.5kHz 50Hz",
            StepCommand {
                sweeps: vec![Sweep::Lin(Lin {
                    swname: SweepName::Param {
                        name: Name(Atom::from("CenterFreq")),
                    },
                    sstart: Number::from(9.5e3),
                    send: Number::from(10.5e3),
                    sinc: Number::from(50.0),
                })]
            }
        )
    }

    #[test]
    fn test_parse_command_subckt() {
        spice_test_ok!(
            ".SUBCKT john 3 2 4 5 PARAM: v=2 b=3 f=5",
            SubcktCommand {
                name: Name(Atom::from("john")),
                pins: vec![
                    Node(Atom::from("3")),
                    Node(Atom::from("2")),
                    Node(Atom::from("4")),
                    Node(Atom::from("5")),
                ],
                optionals: vec![],
                params: vec![
                    Param {
                        name: Name(Atom::from("v")),
                        value: Number::from(2.0),
                    },
                    Param {
                        name: Name(Atom::from("b")),
                        value: Number::from(3.0),
                    },
                    Param {
                        name: Name(Atom::from("f")),
                        value: Number::from(5.0),
                    }
                ],
                texts: vec![],
                instructions: vec![],
            }
        );

        spice_test_ok!(
            ".SUBCKT name 1 2 3 4 OPTIONAL: GND=0 PARAM: v=2 TEXT: name=\"texts\"",
            SubcktCommand {
                name: Name(Atom::from("name")),
                pins: vec![
                    Node(Atom::from("1")),
                    Node(Atom::from("2")),
                    Node(Atom::from("3")),
                    Node(Atom::from("4")),
                ],
                optionals: vec![Pairs {
                    name: Node(Atom::from("GND")),
                    value: Node(Atom::from("0")),
                }],
                params: vec![Param {
                    name: Name(Atom::from("v")),
                    value: Number::from(2.0),
                }],
                texts: vec![Param {
                    name: Name(Atom::from("name")),
                    value: Text(Atom::from("\"texts\"")),
                }],
                instructions: vec![]
            }
        )
    }

    #[test]
    fn test_parse_command_ends() {
        spice_test_ok!(".ENDS", EndsCommand { name: None });

        spice_test_ok!(
            ".ENDS subckt",
            EndsCommand {
                name: Some(Name(Atom::from("subckt")))
            }
        );

        spice_test_err!(".ENDX", EndsCommand);
    }

    #[test]
    fn test_parse_command_temp() {
        spice_test_ok!(
            ".TEMP 2 3 4",
            TempCommand {
                temps: vec![Number::from(2.0), Number::from(3.0), Number::from(4.0),]
            }
        )
    }

    #[test]
    fn test_parse_command_text() {
        spice_test_ok!(
            r#".TEXT a="233" b="tanaka""#,
            TextCommand {
                texts: vec![
                    Param {
                        name: Name(Atom::from("a")),
                        value: Text(Atom::from("\"233\""))
                    },
                    Param {
                        name: Name(Atom::from("b")),
                        value: Text(Atom::from("\"tanaka\""))
                    },
                ]
            }
        )
    }

    #[test]
    fn test_parse_command_tf() {
        spice_test_ok!(
            ".TF V(10) VIN",
            TfCommand {
                output: OutputVariable::Voltage(VoltageVariable::Node {
                    node: ExplicitNode(Node(Atom::from("10"))),
                    suffix: Suffix::None
                }),
                input: Name(Atom::from("VIN")),
            }
        );
    }

    #[test]
    fn test_parse_command_tran() {
        spice_test_ok!(
            ".TRAN 5 10 200 0.1 UIC",
            TranCommand {
                op: false,
                tstep: Number::from(5.0),
                tstop: Number::from(10.0),
                tstart: Some(Number::from(200.0)),
                tmax: Some(Number::from(0.1)),
                params: TranCommandParams {
                    uic: true,
                    skipbp: false
                },
            }
        )
    }

    #[test]
    fn test_parse_command_watch() {
        spice_test_ok!(
            ".WATCH DC V([RESET]) (2.5V,10V)",
            WatchCommand {
                source: ProbeSource::Dc,
                variables: vec![PrintVariable {
                    variable: OutputVariable::Voltage(VoltageVariable::Node {
                        node: ExplicitNode(Node(Atom::from("RESET"))),
                        suffix: Suffix::None
                    }),
                    limits: Some(Limits {
                        lower: Number::from(2.5),
                        upper: Number::from(10.0)
                    }),
                }]
            }
        )
    }

    #[test]
    fn test_parse_command_width() {
        spice_test_ok!(
            ".WIDTH OUT=114",
            WidthCommand {
                width: Number::from(114.0)
            }
        )
    }
}

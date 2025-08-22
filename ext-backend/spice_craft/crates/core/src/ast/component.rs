use serde::Serialize;
use spice_proc_macro::{ExposeNodes, Params, PartialParse, TryParse};

use crate::parse::{
    Element, ElementInfo, ExposeNodes, ParseError, ParseResult, PartialParse, TokenStream, TryParse,
};

use super::{
    expression::Text,
    gain::{CurrentGain, VoltageGain},
    transient::{AcSource, DcSource, Transient},
    ExplicitNode, Name, Node, Number, Param,
};

/// GaAs MES field-effect transistor 砷化镓 MES 场效应晶体管
///
/// - `B<name> <drain node> <gate node> <source node> <model name> [area value]`
///
/// 端子：
/// - `D` 漏极
/// - `G` 栅极
/// - `S` 原极
///
/// e.g.
///
/// - `BIN 100 10 0 GFAST`
/// - `B13 22 14 23 GNOM 2.0`
///
#[derive(Debug, Clone, PartialEq, TryParse, Serialize, ExposeNodes, PartialParse)]
#[grammar(name, drain, gate, source, model, area)]
pub struct BComponent {
    pub name: Name,
    pub drain: Node,
    pub gate: Node,
    pub source: Node,
    pub model: Name,
    pub area: Option<Number>,
}

/// Capacitor 电容
///
/// - `C<name> <(+) node> <(-) node> [model name] <value> [IC=<initial value>]`
///
/// e.g.
///
/// - `CLOAD 15 0 20pF`
/// - `C2 1 2 .2E-12 IC=1.5V`
/// - `CFDBCK 3 33 CMOD 10pF`
#[derive(Debug, Clone, PartialEq, TryParse, Serialize, ExposeNodes, PartialParse)]
#[grammar(name, node1, node2, model, value, params)]
pub struct CComponent {
    /// 名称
    pub name: Name,
    /// 节点1
    pub node1: Node,
    /// 节点2
    pub node2: Node,
    /// 模型
    pub model: Option<Name>,
    /// 电容值
    pub value: Number,
    /// 其他参数
    pub params: CComponentParams,
}

#[derive(Debug, Clone, PartialEq, Params, Serialize, ExposeNodes)]
pub struct CComponentParams {
    #[param(name = "IC")]
    pub ic: Option<Number>,
}

/// Diode 二极管
///
/// - `D<name> <(+) node> <(-) node> <model name> [area value]`
///
/// e.g.
///
/// - `DCLAMP 14 0 DMOD`
/// - `D13 15 17 SWITCH 1.5`
#[derive(Debug, Clone, PartialEq, TryParse, Serialize, ExposeNodes, PartialParse)]
#[grammar(name, node1, node2, model, area)]
pub struct DComponent {
    pub name: Name,
    pub node1: Node,
    pub node2: Node,
    pub model: Name,
    pub area: Option<Number>,
}

/// Voltage-controlled voltage source 电压控制电压源
///
/// - `E<name> <(+) node> <(-) node> <gain>`
#[derive(Debug, Clone, PartialEq, TryParse, Serialize, ExposeNodes, PartialParse)]
#[grammar(name, node1, node2, gain)]
pub struct EComponent {
    pub name: Name,
    pub node1: Node,
    pub node2: Node,
    pub gain: VoltageGain,
}

/// Current-controlled current source 电流控制电流源
///
/// - `F<name> <(+) node> <(-) node> <gain>`
#[derive(Debug, Clone, PartialEq, TryParse, Serialize, ExposeNodes, PartialParse)]
#[grammar(name, node1, node2, gain)]
pub struct FComponent {
    pub name: Name,
    pub node1: Node,
    pub node2: Node,
    pub gain: CurrentGain,
}

/// Voltage-controlled current source 电压控制电流源
///
/// - `G<name> <(+) node> <(-) node> <gain>`
#[derive(Debug, Clone, PartialEq, TryParse, Serialize, ExposeNodes, PartialParse)]
#[grammar(name, node1, node2, gain)]
pub struct GComponent {
    pub name: Name,
    pub node1: Node,
    pub node2: Node,
    pub gain: VoltageGain,
}

/// Current-controlled voltage source 电流控制电压源
///
/// - `H<name> <(+) node> <(-) node> <gain>`
#[derive(Debug, Clone, PartialEq, TryParse, Serialize, ExposeNodes, PartialParse)]
#[grammar(name, node1, node2, gain)]
pub struct HComponent {
    pub name: Name,
    pub node1: Node,
    pub node2: Node,
    pub gain: CurrentGain,
}

/// Independent current source 独立电流源
/// - `I<name> <node1> <node2> [<dc>] [<ac>] [<transient>]`
#[derive(Debug, Clone, PartialEq, TryParse, Serialize, ExposeNodes, PartialParse)]
#[grammar(name, node1, node2, dc, ac, transient)]
pub struct IComponent {
    pub name: Name,
    pub node1: Node,
    pub node2: Node,
    pub dc: Option<DcSource>,
    pub ac: Option<AcSource>,
    pub transient: Option<Transient>,
}

/// Junction field-effect transistor 结型场效应晶体管
///
/// - `J<name> <drain node> <gate node> <source node> <model name> [area value]`
///
/// 端子：
/// - `D` 漏极
/// - `G` 栅极
/// - `S` 原极
///
/// e.g.
/// - `JIN 100 1 0 JFAST`
/// - `J13 22 14 23 JNOM 2.0`
#[derive(Debug, Clone, PartialEq, TryParse, Serialize, ExposeNodes, PartialParse)]
#[grammar(name, drain, gate, source, model, area)]
pub struct JComponent {
    pub name: Name,
    pub drain: Node,
    pub gate: Node,
    pub source: Node,
    pub model: Name,
    pub area: Option<Number>,
}

/// Mutual inductors ( transformer ) 互感器（变压器）
/// - `K<name> <induct1> <induct2> ... <k> [<model> [<size>]]`
/// - `K<name> <transmission1> <transmission2> [Cm=<capacity coupling>] [Lm=<inductive coupling>]`
///
/// TODO: 第二种形式还没写，准备先不写
#[derive(Debug, Clone, PartialEq, TryParse, Serialize, ExposeNodes, PartialParse)]
#[grammar(name, inducts, k, model)]
pub struct KComponent {
    pub name: Name,
    pub inducts: Vec<Name>,
    pub k: Number,
    pub model: Option<(Name, Option<Number>)>,
}

/// Inductor 电感
/// - `L<name> <(+) node> <(-) node> [model name] <value> [IC=<initial value>]`
#[derive(Debug, Clone, PartialEq, TryParse, Serialize, ExposeNodes, PartialParse)]
#[grammar(name, node1, node2, model, value, params)]
pub struct LComponent {
    pub name: Name,
    pub node1: Node,
    pub node2: Node,
    pub model: Option<Name>,
    pub value: Number,
    pub params: LComponentParams,
}

#[derive(Debug, Clone, PartialEq, Params, Serialize, ExposeNodes)]
pub struct LComponentParams {
    #[param(name = "IC")]
    pub ic: Option<Number>,
}

/// MOS field-effect transistor MOS 场效应晶体管
///
/// - `M<name> <drain node> <gate node> <source node>`
/// - `+ <bulk/substrate node> <model name>`
/// - `+ [L=<value>] [W=<value>]`
/// - `+ [AD=<value>] [AS=<value>]`
/// - `+ [PD=<value>] [PS=<value>]`
/// - `+ [NRD=<value>] [NRS=<value>]`
/// - `+ [NRG=<value>] [NRB=<value>]`
/// - `+ [M=<value>] [N=<value>]`
///
/// 端子：
/// - `D` 漏极
/// - `G` 栅极
/// - `S` 原极
/// - `B` 基极
///
/// e.g.
/// - `M1 14 2 13 0 PNOM L=25u W=12u`
/// - `M13 15 3 0 0 PSTRONG`
/// - `M16 17 3 0 0 PSTRONG M=2`
/// - `M28 0 2 100 100 NWEAK L=33u W=12u AD=288p AS=288p PD=60u PS=60u NRD=14 NRS=24 NRG=10`
#[derive(Debug, Clone, PartialEq, TryParse, Serialize, ExposeNodes, PartialParse)]
#[grammar(name, drain, gate, source, bulk, model, params)]
pub struct MComponent {
    pub name: Name,
    pub drain: Node,
    pub gate: Node,
    pub source: Node,
    pub bulk: Node,
    pub model: Name,
    pub params: MComponentParams,
}

#[derive(Debug, Clone, PartialEq, Params, Serialize, ExposeNodes)]
pub struct MComponentParams {
    #[param(name = "L")]
    pub l: Option<Number>,
    #[param(name = "W")]
    pub w: Option<Number>,
    #[param(name = "AD")]
    pub ad: Option<Number>,
    #[param(name = "AS")]
    pub as_: Option<Number>,
    #[param(name = "PD")]
    pub pd: Option<Number>,
    #[param(name = "PS")]
    pub ps: Option<Number>,
    #[param(name = "NRD")]
    pub nrd: Option<Number>,
    #[param(name = "NRS")]
    pub nrs: Option<Number>,
    #[param(name = "NRG")]
    pub nrg: Option<Number>,
    #[param(name = "NRB")]
    pub nrb: Option<Number>,
    #[param(name = "M")]
    pub m: Option<Number>,
    #[param(name = "N")]
    pub n: Option<Number>,
}

/// Bipolar junction transistor 双极结型晶体管
///
/// - `Q<name> <collector node> <base node> <emitter node>`
/// - `+ [substrate node] <model name> [area value]`
///
/// 端子：
/// - `C` 集电极
/// - `B` 基极
/// - `E` 发射极
/// - `S` 衬底
///
/// e.g.
/// - `Q1 14 2 13 PNPNOM`
/// - `Q13 15 3 0 1 NPNSTRONG 1.5`
/// - `Q7 VC 5 12 [SUB] LATPNP`
#[derive(Debug, Clone, PartialEq, TryParse, Serialize, ExposeNodes, PartialParse)]
#[grammar(name, collector, base, emitter, substrate, model, area)]
pub struct QComponent {
    pub name: Name,
    pub collector: Node,
    pub base: Node,
    pub emitter: Node,
    pub substrate: Option<ExplicitNode>,
    pub model: Name,
    pub area: Option<Number>,
}

/// Resistor 电阻
/// - `R<name> <(+) node> <(-) node> [model name] <value> [TC = <TC1> [,<TC2>]]`
///
/// e.g.
/// - `RLOAD 15 0 2K`
/// - `R2 1 2 2.4E4 TC=.015,-.003`
/// - `RFDBCK 3 33 RMOD 10K`
#[derive(Debug, Clone, PartialEq, TryParse, Serialize, ExposeNodes, PartialParse)]
#[grammar(name, node1, node2, model, value, params)]
pub struct RComponent {
    pub name: Name,
    pub node1: Node,
    pub node2: Node,
    pub model: Option<Name>,
    pub value: Number,
    pub params: RComponentParams,
}

#[derive(Debug, Clone, PartialEq, Params, Serialize, ExposeNodes)]
pub struct RComponentParams {
    #[param(name = "TC")]
    pub tc: Option<(Number, Option<Number>)>,
}

/// Voltage-controlled switch 电压控制开关
/// - `S<name> <(+) switch node> <(-) switch node>`
/// - `+ <(+) controlling node> <(-) controlling node>`
/// - `+ <model name>`
#[derive(Debug, Clone, PartialEq, TryParse, Serialize, ExposeNodes, PartialParse)]
#[grammar(name, snode1, snode2, cnode1, cnode2, model)]
pub struct SComponent {
    pub name: Name,
    pub snode1: Node,
    pub snode2: Node,
    pub cnode1: Node,
    pub cnode2: Node,
    pub model: Name,
}

/// Transmission line 输电线路
/// - `T<name> <A port (+) node> <A port (-) node>`
/// - `+ <B port (+) node> <B port (-) node>`
/// - `+ [model name]`
/// - `+ Z0=<value> [TD=<value>] [F=<value> [NL=<value>]]`
/// - `+ IC= <near voltage> <near current> <far voltage> <far current>`
///
/// or
/// - `T<name> <A port (+) node> <A port (-) node>`
/// - `+ <B port (+) node> <B port (-) node>`
/// - `+ [ <model name> [electrical length value] ]`
/// - `+ LEN=<value> R=<value> L=<value>`
/// - `+ G=<value> C=<value>`
///
///
#[derive(Debug, Clone, PartialEq, TryParse, Serialize, ExposeNodes, PartialParse)]
#[grammar(name, anode1, anode2, bnode1, bnode2, model, params)]
pub struct TComponent {
    pub name: Name,
    pub anode1: Node,
    pub anode2: Node,
    pub bnode1: Node,
    pub bnode2: Node,
    pub model: Option<(Name, Option<Number>)>,
    pub params: TComponentParams,
}

#[derive(Debug, Clone, PartialEq, Params, Serialize, ExposeNodes)]
pub struct TComponentParams {
    #[param(name = "Z0")]
    pub z0: Option<Number>,
    #[param(name = "TD")]
    pub td: Option<Number>,
    #[param(name = "F")]
    pub f: Option<Number>,
    #[param(name = "NL")]
    pub nl: Option<Number>,
    #[param(name = "IC")]
    pub ic: Option<(Number, Number, Number, Number)>,
    #[param(name = "LEN")]
    pub len: Option<Number>,
    #[param(name = "R")]
    pub r: Option<Number>,
    #[param(name = "L")]
    pub l: Option<Number>,
    #[param(name = "G")]
    pub g: Option<Number>,
    #[param(name = "C")]
    pub c: Option<Number>,
}

/// Independent voltage source 独立电压源
/// - `V<name> <node1> <node2> [<dc>] [<ac>] [<transient>]`
#[derive(Debug, Clone, PartialEq, TryParse, Serialize, ExposeNodes, PartialParse)]
#[grammar(name, node1, node2, dc, ac, transient)]
pub struct VComponent {
    pub name: Name,
    pub node1: Node,
    pub node2: Node,
    pub dc: Option<DcSource>,
    pub ac: Option<AcSource>,
    pub transient: Option<Transient>,
}

/// Current-controlled switch 电流控制开关
/// - `W<name> <(+) switch node> <(-) switch node> <controlling V device name> <model name>`
#[derive(Debug, Clone, PartialEq, TryParse, Serialize, ExposeNodes, PartialParse)]
#[grammar(name, snode1, snode2, vdevice, model)]
pub struct WComponent {
    pub name: Name,
    pub snode1: Node,
    pub snode2: Node,
    pub vdevice: Name,
    pub model: Name,
}

/// 调用子电路
/// - `X<name> [node]* <subcircuit name> [PARAM: <<name> = <value>>*]`
/// - `+ [TEXT: < <name> = <text value> >* ]`
#[derive(Debug, Clone, PartialEq, Serialize, ExposeNodes)]
pub struct XComponent {
    pub name: Name,
    pub pins: Vec<Node>,
    pub sname: Name,
    pub params: Vec<Param<Number>>,
    pub texts: Vec<Param<Text>>,
}

impl<T> TryParse<XComponent> for T
where
    T: TokenStream,
{
    fn try_parse(&mut self) -> ParseResult<XComponent> {
        let name = self.try_parse()?;
        let mut pins = Vec::new();
        let mut params = Vec::new();
        let mut texts = Vec::new();

        while !self.next_eof() && !self.next_matches("PARAM") && !self.next_matches("TEXT") {
            pins.push(self.try_parse()?);
        }

        let sname = self.try_parse()?;

        if self.matches_consume("PARAM") {
            self.expect(":")?;
            while !self.is_eof() && !self.matches("TEXT") {
                params.push(self.try_parse()?);
            }
        }

        if self.matches_consume("TEXT") {
            self.expect(":")?;
            while !self.is_eof() {
                texts.push(self.try_parse()?);
            }
        }

        Ok(XComponent {
            name,
            pins,
            sname,
            params,
            texts,
        })
    }
}

/// IGBT
/// - `Z<name> <collector> <gate> <emitter> <model name>`
/// - `[AREA=<value>] [WB=<value>] [AGD=<value>]`
/// - `[KP=<value>] [TAU=<value>]`
///
/// 端子：
/// - `G` 栅极
/// - `C` 集电极
/// - `E` 发射极
///
/// e.g.
/// - `ZDRIVE 1 4 2 IGBTA AREA=10.1u WB=91u AGD=5.1u KP=0.381`
/// - `Z231 3 2 9 IGBT27`
#[derive(Debug, Clone, PartialEq, TryParse, Serialize, ExposeNodes, PartialParse)]
#[grammar(name, collector, gate, emitter, model, params)]
pub struct ZComponent {
    pub name: Name,
    pub collector: Node,
    pub gate: Node,
    pub emitter: Node,
    pub model: Name,
    pub params: ZComponentParams,
}

#[derive(Debug, Clone, PartialEq, Params, Serialize)]
pub struct ZComponentParams {
    #[param(name = "AREA")]
    pub area: Option<Number>,
    #[param(name = "WB")]
    pub wb: Option<Number>,
    #[param(name = "AGD")]
    pub agd: Option<Number>,
    #[param(name = "KP")]
    pub kp: Option<Number>,
    #[param(name = "TAU")]
    pub tau: Option<Number>,
}

/// 组件声明
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "type")]
pub enum Component {
    B(BComponent),
    C(CComponent),
    D(DComponent),
    E(EComponent),
    F(FComponent),
    G(GComponent),
    H(HComponent),
    I(IComponent),
    J(JComponent),
    K(KComponent),
    L(LComponent),
    M(MComponent),
    Q(QComponent),
    R(RComponent),
    S(SComponent),
    T(TComponent),
    V(VComponent),
    W(WComponent),
    X(XComponent),
    Z(ZComponent),
}

impl ExposeNodes for Component {
    fn nodes(&self) -> Vec<Node> {
        match self {
            Component::B(c) => c.nodes(),
            Component::C(c) => c.nodes(),
            Component::D(c) => c.nodes(),
            Component::E(c) => c.nodes(),
            Component::F(c) => c.nodes(),
            Component::G(c) => c.nodes(),
            Component::H(c) => c.nodes(),
            Component::I(c) => c.nodes(),
            Component::J(c) => c.nodes(),
            Component::K(c) => c.nodes(),
            Component::L(c) => c.nodes(),
            Component::M(c) => c.nodes(),
            Component::Q(c) => c.nodes(),
            Component::R(c) => c.nodes(),
            Component::S(c) => c.nodes(),
            Component::T(c) => c.nodes(),
            Component::V(c) => c.nodes(),
            Component::W(c) => c.nodes(),
            Component::X(c) => c.nodes(),
            Component::Z(c) => c.nodes(),
        }
    }
}

impl<T> TryParse<Component> for T
where
    T: TokenStream,
{
    fn try_parse(&mut self) -> ParseResult<Component> {
        let token = self.token()?;
        match &token.to_uppercase()[..] {
            name if name.starts_with("B") => Ok(Component::B(self.try_parse()?)),
            name if name.starts_with("C") => Ok(Component::C(self.try_parse()?)),
            name if name.starts_with("D") => Ok(Component::D(self.try_parse()?)),
            name if name.starts_with("E") => Ok(Component::E(self.try_parse()?)),
            name if name.starts_with("F") => Ok(Component::F(self.try_parse()?)),
            name if name.starts_with("G") => Ok(Component::G(self.try_parse()?)),
            name if name.starts_with("H") => Ok(Component::H(self.try_parse()?)),
            name if name.starts_with("I") => Ok(Component::I(self.try_parse()?)),
            name if name.starts_with("J") => Ok(Component::J(self.try_parse()?)),
            name if name.starts_with("K") => Ok(Component::K(self.try_parse()?)),
            name if name.starts_with("L") => Ok(Component::L(self.try_parse()?)),
            name if name.starts_with("M") => Ok(Component::M(self.try_parse()?)),
            name if name.starts_with("Q") => Ok(Component::Q(self.try_parse()?)),
            name if name.starts_with("R") => Ok(Component::R(self.try_parse()?)),
            name if name.starts_with("S") => Ok(Component::S(self.try_parse()?)),
            name if name.starts_with("T") => Ok(Component::T(self.try_parse()?)),
            name if name.starts_with("V") => Ok(Component::V(self.try_parse()?)),
            name if name.starts_with("W") => Ok(Component::W(self.try_parse()?)),
            name if name.starts_with("X") => Ok(Component::X(self.try_parse()?)),
            name if name.starts_with("Z") => Ok(Component::Z(self.try_parse()?)),
            _ => Err(ParseError {
                reason: format!("Expect `{}' to be a Component name", token.raw),
                position: Some(token.column),
            }),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum ComponentPartial {
    B(BComponentPartial),
    C(CComponentPartial),
    D(DComponentPartial),
    E(EComponentPartial),
    F(FComponentPartial),
    G(GComponentPartial),
    H(HComponentPartial),
    I(IComponentPartial),
    J(JComponentPartial),
    K(KComponentPartial),
    L(LComponentPartial),
    M(MComponentPartial),
    Q(QComponentPartial),
    R(RComponentPartial),
    S(SComponentPartial),
    T(TComponentPartial),
    V(VComponentPartial),
    W(WComponentPartial),
    //X(XComponentPartial),
    Z(ZComponentPartial),
    Unknown,
}

impl<T> PartialParse<ComponentPartial> for T
where
    T: TokenStream,
{
    fn try_partial(&mut self) -> ParseResult<ComponentPartial> {
        let token = self.token()?;
        match &token.to_uppercase()[..] {
            name if name.starts_with("B") => Ok(ComponentPartial::B(self.try_partial()?)),
            name if name.starts_with("C") => Ok(ComponentPartial::C(self.try_partial()?)),
            name if name.starts_with("D") => Ok(ComponentPartial::D(self.try_partial()?)),
            name if name.starts_with("E") => Ok(ComponentPartial::E(self.try_partial()?)),
            name if name.starts_with("F") => Ok(ComponentPartial::F(self.try_partial()?)),
            name if name.starts_with("G") => Ok(ComponentPartial::G(self.try_partial()?)),
            name if name.starts_with("H") => Ok(ComponentPartial::H(self.try_partial()?)),
            name if name.starts_with("I") => Ok(ComponentPartial::I(self.try_partial()?)),
            name if name.starts_with("J") => Ok(ComponentPartial::J(self.try_partial()?)),
            name if name.starts_with("K") => Ok(ComponentPartial::K(self.try_partial()?)),
            name if name.starts_with("L") => Ok(ComponentPartial::L(self.try_partial()?)),
            name if name.starts_with("M") => Ok(ComponentPartial::M(self.try_partial()?)),
            name if name.starts_with("Q") => Ok(ComponentPartial::Q(self.try_partial()?)),
            name if name.starts_with("R") => Ok(ComponentPartial::R(self.try_partial()?)),
            name if name.starts_with("S") => Ok(ComponentPartial::S(self.try_partial()?)),
            name if name.starts_with("T") => Ok(ComponentPartial::T(self.try_partial()?)),
            name if name.starts_with("V") => Ok(ComponentPartial::V(self.try_partial()?)),
            name if name.starts_with("W") => Ok(ComponentPartial::W(self.try_partial()?)),
            //name if name.starts_with("X") => Ok(ComponentPartial::X(self.try_partial()?)),
            name if name.starts_with("Z") => Ok(ComponentPartial::Z(self.try_partial()?)),
            _ => Err(ParseError {
                reason: format!("Expect `{}' to be a Component name", token.raw),
                position: Some(token.column),
            }),
        }
    }

    fn info(&mut self) -> ParseResult<(ComponentPartial, Vec<Element>)> {
        let token = self.token()?;
        match &token.to_uppercase()[..] {
            name if name.starts_with("B") => {
                Ok((self.info().map(|(x, y)| (ComponentPartial::B(x), y)))?)
            }
            name if name.starts_with("C") => {
                Ok((self.info().map(|(x, y)| (ComponentPartial::C(x), y)))?)
            }

            name if name.starts_with("D") => {
            Ok((self.info().map(|(x, y)| (ComponentPartial::D(x), y)))?)
            }

            name if name.starts_with("E") => {
                Ok((self.info().map(|(x, y)| (ComponentPartial::E(x), y)))?)
            }

            name if name.starts_with("F") => {
                Ok((self.info().map(|(x, y)| (ComponentPartial::F(x), y)))?)
            }

            name if name.starts_with("G") => {
                Ok((self.info().map(|(x, y)| (ComponentPartial::G(x), y)))?)
            }

            name if name.starts_with("H") => {
                Ok((self.info().map(|(x, y)| (ComponentPartial::H(x), y)))?)
            }

            name if name.starts_with("I") => {
                Ok((self.info().map(|(x, y)| (ComponentPartial::I(x), y)))?)
            }

            name if name.starts_with("J") => {
                Ok((self.info().map(|(x, y)| (ComponentPartial::J(x), y)))?)
            }

            name if name.starts_with("K") => {
                Ok((self.info().map(|(x, y)| (ComponentPartial::K(x), y)))?)
            }

            name if name.starts_with("L") => {
                Ok((self.info().map(|(x, y)| (ComponentPartial::L(x), y)))?)
            }

            name if name.starts_with("M") => {
                Ok((self.info().map(|(x, y)| (ComponentPartial::M(x), y)))?)
            }

            name if name.starts_with("Q") => {
                Ok((self.info().map(|(x, y)| (ComponentPartial::Q(x), y)))?)
            }

            name if name.starts_with("R") => {
                Ok((self.info().map(|(x, y)| (ComponentPartial::R(x), y)))?)
            }

            name if name.starts_with("S") => {
                Ok((self.info().map(|(x, y)| (ComponentPartial::S(x), y)))?)
            }

            name if name.starts_with("T") => {
                Ok((self.info().map(|(x, y)| (ComponentPartial::T(x), y)))?)
            }

            name if name.starts_with("V") => {
                Ok((self.info().map(|(x, y)| (ComponentPartial::V(x), y)))?)
            }

            name if name.starts_with("W") => {
                Ok((self.info().map(|(x, y)| (ComponentPartial::W(x), y)))?)
            }

            name if name.starts_with("Z") => {
                Ok((self.info().map(|(x, y)| (ComponentPartial::Z(x), y)))?)
            }

            _ => Err(ParseError {
                reason: format!("Expect `{}' to be a Component name", token.raw),
                position: Some(token.column),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        ast::{
            gain::{PolyCurrentGain, PolyVoltageGain, SimpleCurrentGain, SimpleVoltageGain},
            transient::Sin,
            Atom,
        },
        parse::SpiceLineParser,
    };

    use super::*;

    #[test]
    fn test_parse_component_b() {
        let tokens = vec![
            Atom::from("B2"),
            Atom::from("1"),
            Atom::from("2"),
            Atom::from("3"),
            Atom::from("BBSS"),
            Atom::from("114.514"),
        ];
        let mut parser = SpiceLineParser::new(&tokens);
        assert_eq!(
            parser.try_parse(),
            Ok(Component::B(BComponent {
                name: Name(Atom::from("B2")),
                drain: Node(Atom::from("1")),
                gate: Node(Atom::from("2")),
                source: Node(Atom::from("3")),
                model: Name(Atom::from("BBSS")),
                area: Some(Number::from(114.514)),
            }))
        );
        assert!(parser.is_eof());
    }

    #[test]
    fn test_partparse_component_b() {
        let tokens = vec![Atom::from("B2")];
        let mut parser = SpiceLineParser::new(&tokens);
        let partial: CComponentPartial = parser.try_partial().unwrap();
        assert_eq!(partial.name, Some(Name(Atom::from("B2"))));
    }

    #[test]
    fn test_parse_component_c() {
        let tokens = vec![
            Atom::from("C1"),
            Atom::from("1"),
            Atom::from("2"),
            Atom::from("1.0"),
        ];
        let mut parser = SpiceLineParser::new(&tokens);
        assert_eq!(
            parser.try_parse(),
            Ok(Component::C(CComponent {
                name: Name(Atom::from("C1")),
                node1: Node(Atom::from("1")),
                node2: Node(Atom::from("2")),
                model: None,
                value: Number::from(1.0),
                params: CComponentParams { ic: None },
            }))
        );
        assert!(parser.is_eof());

        let tokens = vec![
            Atom::from("C9"),
            Atom::from("1"),
            Atom::from("2"),
            Atom::from("COC"),
            Atom::from("1.0"),
            Atom::from("IC"),
            Atom::from("="),
            Atom::from("2.0"),
        ];
        let mut parser = SpiceLineParser::new(&tokens);
        let a = parser.try_parse();
        assert_eq!(
            a,
            Ok(Component::C(CComponent {
                name: Name(Atom::from("C9")),
                node1: Node(Atom::from("1")),
                node2: Node(Atom::from("2")),
                model: Some(Name(Atom::from("COC"))),
                value: Number::from(1.0),
                params: CComponentParams {
                    ic: Some(Number::from(2.0))
                },
            }))
        );
        assert!(parser.is_eof());
        println!("{:#?}",a);
    }

    #[test]
    fn test_parse_component_d() {
        let tokens = vec![
            Atom::from("D3"),
            Atom::from("1"),
            Atom::from("2"),
            Atom::from("DBUS"),
            Atom::from("114.514"),
        ];
        let mut parser = SpiceLineParser::new(&tokens);
        assert_eq!(
            parser.try_parse(),
            Ok(Component::D(DComponent {
                name: Name(Atom::from("D3")),
                node1: Node(Atom::from("1")),
                node2: Node(Atom::from("2")),
                model: Name(Atom::from("DBUS")),
                area: Some(Number::from(114.514)),
            }))
        );
        assert!(parser.is_eof());
    }

    #[test]
    fn test_parse_component_e() {
        let tokens = vec![
            Atom::from("E1"),
            Atom::from("1"),
            Atom::from("2"),
            Atom::from("5"),
            Atom::from("9"),
            Atom::from("10"),
        ];
        let mut parser = SpiceLineParser::new(&tokens);
        assert_eq!(
            parser.try_parse(),
            Ok(Component::E(EComponent {
                name: Name(Atom::from("E1")),
                node1: Node(Atom::from("1")),
                node2: Node(Atom::from("2")),
                gain: VoltageGain::Simple(SimpleVoltageGain {
                    node1: Node(Atom::from("5")),
                    node2: Node(Atom::from("9")),
                    gain: Number::from(10.0),
                }),
            }))
        );
        assert!(parser.is_eof());
    }

    #[test]
    fn test_parse_component_f() {
        let tokens = vec![
            Atom::from("F1"),
            Atom::from("1"),
            Atom::from("2"),
            Atom::from("V1"),
            Atom::from("10"),
        ];
        let mut parser = SpiceLineParser::new(&tokens);
        assert_eq!(
            parser.try_parse(),
            Ok(Component::F(FComponent {
                name: Name(Atom::from("F1")),
                node1: Node(Atom::from("1")),
                node2: Node(Atom::from("2")),
                gain: CurrentGain::Simple(SimpleCurrentGain {
                    vdevice: Name(Atom::from("V1")),
                    gain: Number::from(10.0),
                }),
            }))
        );
        assert!(parser.is_eof());
    }

    #[test]
    fn test_parse_component_g() {
        let tokens = vec![
            Atom::from("G1"),
            Atom::from("1"),
            Atom::from("2"),
            Atom::from("POLY"),
            Atom::from("("),
            Atom::from("1"),
            Atom::from(")"),
            Atom::from("3"),
            Atom::from("4"),
            Atom::from("5"),
        ];
        let mut parser = SpiceLineParser::new(&tokens);
        assert_eq!(
            parser.try_parse(),
            Ok(Component::G(GComponent {
                name: Name(Atom::from("G1")),
                node1: Node(Atom::from("1")),
                node2: Node(Atom::from("2")),
                gain: VoltageGain::Poly(PolyVoltageGain {
                    n: 1,
                    nodes: vec![(Node(Atom::from("3")), Node(Atom::from("4")))],
                    coefficients: vec![Number::from(5.0)],
                }),
            }))
        );
        assert!(parser.is_eof());
    }

    #[test]
    fn test_parse_component_h() {
        let tokens = vec![
            Atom::from("H1"),
            Atom::from("1"),
            Atom::from("2"),
            Atom::from("POLY"),
            Atom::from("("),
            Atom::from("1"),
            Atom::from(")"),
            Atom::from("V2"),
            Atom::from("4"),
            Atom::from("5"),
        ];
        let mut parser = SpiceLineParser::new(&tokens);
        assert_eq!(
            parser.try_parse(),
            Ok(Component::H(HComponent {
                name: Name(Atom::from("H1")),
                node1: Node(Atom::from("1")),
                node2: Node(Atom::from("2")),
                gain: CurrentGain::Poly(PolyCurrentGain {
                    n: 1,
                    vdevices: vec![Name(Atom::from("V2"))],
                    coefficients: vec![Number::from(4.0), Number::from(5.0)],
                }),
            }))
        );
        assert!(parser.is_eof());
    }

    #[test]

    fn test_parse_component_i() {
        let tokens = vec![
            Atom::from("I1"),
            Atom::from("1"),
            Atom::from("2"),
            Atom::from("DC"),
            Atom::from("114514"),
        ];
        let mut parser = SpiceLineParser::new(&tokens);
        assert_eq!(
            parser.try_parse(),
            Ok(Component::I(IComponent {
                name: Name(Atom::from("I1")),
                node1: Node(Atom::from("1")),
                node2: Node(Atom::from("2")),
                dc: Some(DcSource {
                    dc: true,
                    value: Number::from(114514.0)
                }),
                ac: None,
                transient: None,
            }))
        );
        assert!(parser.is_eof());

        let tokens = vec![
            Atom::from("I9"),
            Atom::from("1"),
            Atom::from("3"),
            Atom::from("DC"),
            Atom::from("114514"),
            Atom::from("AC"),
            Atom::from("1919"),
            Atom::from("810"),
            Atom::from("SIN"),
            Atom::from("("),
            Atom::from("1"),
            Atom::from("1"),
            Atom::from("4"),
            Atom::from(")"),
        ];
        let mut parser = SpiceLineParser::new(&tokens);
        assert_eq!(
            parser.try_parse(),
            Ok(Component::I(IComponent {
                name: Name(Atom::from("I9")),
                node1: Node(Atom::from("1")),
                node2: Node(Atom::from("3")),
                dc: Some(DcSource {
                    dc: true,
                    value: Number::from(114514.0)
                }),
                ac: Some(AcSource {
                    magnitude: Number::from(1919.0),
                    phase: Some(Number::from(810.0)),
                }),
                transient: Some(Transient::Sin(Sin {
                    v0: Number::from(1.0),
                    vampl: Number::from(1.0),
                    freq: Some(Number::from(4.0)),
                    td: None,
                    alpha: None,
                    theta: None,
                })),
            }))
        );
        assert!(parser.is_eof());
    }

    #[test]
    fn test_parse_component_j() {
        let tokens = vec![
            Atom::from("J3"),
            Atom::from("1"),
            Atom::from("2"),
            Atom::from("3"),
            Atom::from("JetBrains"),
            Atom::from("114.514"),
        ];
        let mut parser = SpiceLineParser::new(&tokens);
        assert_eq!(
            parser.try_parse(),
            Ok(Component::J(JComponent {
                name: Name(Atom::from("J3")),
                drain: Node(Atom::from("1")),
                gate: Node(Atom::from("2")),
                source: Node(Atom::from("3")),
                model: Name(Atom::from("JetBrains")),
                area: Some(Number::from(114.514)),
            }))
        );
        assert!(parser.is_eof());
    }

    #[test]
    fn test_partparse_component_j() {
        let tokens = vec![Atom::from("JON"), Atom::from("2"), Atom::from("21")];
        let mut parser = SpiceLineParser::new(&tokens);
        let partial: JComponentPartial = parser.try_partial().unwrap();
        assert_eq!(partial.name, Some(Name(Atom::from("JON"))));
        println!("{:#?}", partial);
    }

    #[test]
    fn test_parse_component_k() {
        let tokens = vec![
            Atom::from("K3"),
            Atom::from("I3"),
            Atom::from("I9"),
            Atom::from("3"),
            Atom::from("KOP"),
        ];
        let mut parser = SpiceLineParser::new(&tokens);
        assert_eq!(
            parser.try_parse(),
            Ok(Component::K(KComponent {
                name: Name(Atom::from("K3")),
                inducts: vec![Name(Atom::from("I3")), Name(Atom::from("I9"))],
                k: Number::from(3.0),
                model: Some((Name(Atom::from("KOP")), None)),
            }))
        );
        assert!(parser.is_eof());
    }

    #[test]
    fn test_parse_component_l() {
        let tokens = vec![
            Atom::from("L1"),
            Atom::from("1"),
            Atom::from("2"),
            Atom::from("1.0"),
        ];
        let mut parser = SpiceLineParser::new(&tokens);
        assert_eq!(
            parser.try_parse(),
            Ok(Component::L(LComponent {
                name: Name(Atom::from("L1")),
                node1: Node(Atom::from("1")),
                node2: Node(Atom::from("2")),
                model: None,
                value: Number::from(1.0),
                params: LComponentParams { ic: None }
            }))
        );
        assert!(parser.is_eof());

        let tokens = vec![
            Atom::from("L9"),
            Atom::from("1"),
            Atom::from("2"),
            Atom::from("LOC"),
            Atom::from("1.0"),
            Atom::from("IC"),
            Atom::from("="),
            Atom::from("2.0"),
        ];
        let mut parser = SpiceLineParser::new(&tokens);
        assert_eq!(
            parser.try_parse(),
            Ok(Component::L(LComponent {
                name: Name(Atom::from("L9")),
                node1: Node(Atom::from("1")),
                node2: Node(Atom::from("2")),
                model: Some(Name(Atom::from("LOC"))),
                value: Number::from(1.0),
                params: LComponentParams {
                    ic: Some(Number::from(2.0)),
                }
            }))
        );
        assert!(parser.is_eof());
    }

    #[test]
    fn test_parse_component_m() {
        let tokens = vec![
            Atom::from("M3"),
            Atom::from("2"),
            Atom::from("3"),
            Atom::from("5"),
            Atom::from("0"),
            Atom::from("MMM"),
        ];
        let mut parser = SpiceLineParser::new(&tokens);
        assert_eq!(
            parser.try_parse(),
            Ok(Component::M(MComponent {
                name: Name(Atom::from("M3")),
                drain: Node(Atom::from("2")),
                gate: Node(Atom::from("3")),
                source: Node(Atom::from("5")),
                bulk: Node(Atom::from("0")),
                model: Name(Atom::from("MMM")),
                params: MComponentParams {
                    l: None,
                    w: None,
                    ad: None,
                    as_: None,
                    pd: None,
                    ps: None,
                    nrd: None,
                    nrs: None,
                    nrg: None,
                    nrb: None,
                    m: None,
                    n: None,
                }
            }))
        );
        assert!(parser.is_eof());
    }

    #[test]
    fn test_parse_component_q() {
        let tokens = "Q7 VC 5 12 [ SUB ] LATPNP"
            .split_ascii_whitespace()
            .map(|x| Atom::from(x))
            .collect();
        let mut parser = SpiceLineParser::new(&tokens);
        assert_eq!(
            parser.try_parse(),
            Ok(Component::Q(QComponent {
                name: Name(Atom::from("Q7")),
                collector: Node(Atom::from("VC")),
                base: Node(Atom::from("5")),
                emitter: Node(Atom::from("12")),
                substrate: Some(ExplicitNode(Node(Atom::from("SUB")))),
                model: Name(Atom::from("LATPNP")),
                area: None,
            }))
        );
        assert!(parser.is_eof());
    }

    #[test]
    fn test_parse_component_r() {
        let tokens = vec![
            Atom::from("R1"),
            Atom::from("1"),
            Atom::from("2"),
            Atom::from("1.0M"),
        ];
        let mut parser = SpiceLineParser::new(&tokens);
        assert_eq!(
            parser.try_parse(),
            Ok(Component::R(RComponent {
                name: Name(Atom::from("R1")),
                node1: Node(Atom::from("1")),
                node2: Node(Atom::from("2")),
                model: None,
                value: Number::from(1e-3),
                params: RComponentParams { tc: None }
            }))
        );
        assert!(parser.is_eof());

        let tokens = vec![
            Atom::from("R2"),
            Atom::from("1"),
            Atom::from("2"),
            Atom::from("ROC"),
            Atom::from("1.0"),
        ];
        let mut parser = SpiceLineParser::new(&tokens);
        assert_eq!(
            parser.try_parse(),
            Ok(Component::R(RComponent {
                name: Name(Atom::from("R2")),
                node1: Node(Atom::from("1")),
                node2: Node(Atom::from("2")),
                model: Some(Name(Atom::from("ROC"))),
                value: Number::from(1.0),
                params: RComponentParams { tc: None }
            }))
        );
        assert!(parser.is_eof());
    }

    #[test]
    fn test_partparse_component_r() {
        let tokens = vec![Atom::from("R1"), Atom::from("1"), Atom::from("2")];
        let mut parser = SpiceLineParser::new(&tokens);
        let partial: CComponentPartial = parser.try_partial().unwrap();
        println!("{}\n\n{:#?}", "R1 1 2", partial);
        assert_eq!(partial.name, Some(Name(Atom::from("R1"))));
    }

    #[test]
    fn test_info_component_r() {
        let tokens = vec![Atom::from("R1"), Atom::from("1"), Atom::from("2")];
        let mut parser = SpiceLineParser::new(&tokens);
        let (partial, tokens): (CComponentPartial,_) = parser.info().unwrap();
        println!("{:#?} {:#?}",partial, tokens);
    }

    #[test]
    fn test_parse_component_s() {
        let tokens = vec![
            Atom::from("S3"),
            Atom::from("1"),
            Atom::from("2"),
            Atom::from("3"),
            Atom::from("4"),
            Atom::from("SOP"),
        ];
        let mut parser = SpiceLineParser::new(&tokens);
        assert_eq!(
            parser.try_parse(),
            Ok(Component::S(SComponent {
                name: Name(Atom::from("S3")),
                snode1: Node(Atom::from("1")),
                snode2: Node(Atom::from("2")),
                cnode1: Node(Atom::from("3")),
                cnode2: Node(Atom::from("4")),
                model: Name(Atom::from("SOP")),
            }))
        );
        assert!(parser.is_eof());
    }

    #[test]
    #[ignore]
    // FIXME: 语法有歧义
    fn test_parse_component_t() {
        let tokens = vec![
            Atom::from("T1"),
            Atom::from("1"),
            Atom::from("2"),
            Atom::from("3"),
            Atom::from("4"),
            Atom::from("Z0"),
            Atom::from("="),
            Atom::from("5"),
        ];
        let mut parser = SpiceLineParser::new(&tokens);
        assert_eq!(
            parser.try_parse(),
            Ok(Component::T(TComponent {
                name: Name(Atom::from("T1")),
                anode1: Node(Atom::from("1")),
                anode2: Node(Atom::from("2")),
                bnode1: Node(Atom::from("3")),
                bnode2: Node(Atom::from("4")),
                model: None,
                params: TComponentParams {
                    z0: Some(Number::from(5.0)),
                    td: None,
                    f: None,
                    nl: None,
                    ic: None,
                    len: None,
                    r: None,
                    l: None,
                    g: None,
                    c: None
                }
            }))
        );
        assert!(parser.is_eof());

        let tokens = vec![
            Atom::from("T1"),
            Atom::from("1"),
            Atom::from("2"),
            Atom::from("3"),
            Atom::from("4"),
            Atom::from("Z0"),
            Atom::from("="),
            Atom::from("5"),
            Atom::from("TD"),
            Atom::from("="),
            Atom::from("6"),
            Atom::from("F"),
            Atom::from("="),
            Atom::from("7"),
            Atom::from("NL"),
            Atom::from("="),
            Atom::from("8"),
        ];
        let mut parser = SpiceLineParser::new(&tokens);
        assert_eq!(
            parser.try_parse(),
            Ok(Component::T(TComponent {
                name: Name(Atom::from("T1")),
                anode1: Node(Atom::from("1")),
                anode2: Node(Atom::from("2")),
                bnode1: Node(Atom::from("3")),
                bnode2: Node(Atom::from("4")),
                model: None,
                params: TComponentParams {
                    z0: Some(Number::from(5.0)),
                    td: Some(Number::from(6.0)),
                    f: Some(Number::from(7.0)),
                    nl: Some(Number::from(8.0)),
                    ic: None,
                    len: None,
                    r: None,
                    l: None,
                    g: None,
                    c: None,
                }
            }))
        );
        assert!(parser.is_eof());
    }

    #[test]
    fn test_parse_component_v() {
        let tokens = vec![
            Atom::from("V1"),
            Atom::from("1"),
            Atom::from("2"),
            Atom::from("DC"),
            Atom::from("1.0"),
        ];
        let mut parser = SpiceLineParser::new(&tokens);
        assert_eq!(
            parser.try_parse(),
            Ok(Component::V(VComponent {
                name: Name(Atom::from("V1")),
                node1: Node(Atom::from("1")),
                node2: Node(Atom::from("2")),
                dc: Some(DcSource {
                    dc: true,
                    value: Number::from(1.0)
                }),
                ac: None,
                transient: None,
            }))
        );
        assert!(parser.is_eof());

        let tokens = vec![
            Atom::from("V9"),
            Atom::from("1"),
            Atom::from("3"),
            Atom::from("DC"),
            Atom::from("1.0"),
            Atom::from("AC"),
            Atom::from("2.0"),
            Atom::from("3.0"),
            Atom::from("SIN"),
            Atom::from("("),
            Atom::from("1"),
            Atom::from("2"),
            Atom::from("4"),
            Atom::from(")"),
        ];
        let mut parser = SpiceLineParser::new(&tokens);
        assert_eq!(
            parser.try_parse(),
            Ok(Component::V(VComponent {
                name: Name(Atom::from("V9")),
                node1: Node(Atom::from("1")),
                node2: Node(Atom::from("3")),
                dc: Some(DcSource {
                    dc: true,
                    value: Number::from(1.0)
                }),
                ac: Some(AcSource {
                    magnitude: Number::from(2.0),
                    phase: Some(Number::from(3.0)),
                }),
                transient: Some(Transient::Sin(Sin {
                    v0: Number::from(1.0),
                    vampl: Number::from(2.0),
                    freq: Some(Number::from(4.0)),
                    td: None,
                    alpha: None,
                    theta: None,
                })),
            }))
        );
        assert!(parser.is_eof());
    }

    #[test]
    fn test_parse_component_w() {
        let tokens = vec![
            Atom::from("W1"),
            Atom::from("1"),
            Atom::from("2"),
            Atom::from("V22"),
            Atom::from("WOP"),
        ];
        let mut parser = SpiceLineParser::new(&tokens);
        assert_eq!(
            parser.try_parse(),
            Ok(Component::W(WComponent {
                name: Name(Atom::from("W1")),
                snode1: Node(Atom::from("1")),
                snode2: Node(Atom::from("2")),
                vdevice: Name(Atom::from("V22")),
                model: Name(Atom::from("WOP")),
            }))
        );
        assert!(parser.is_eof());
    }

    #[test]
    fn test_parse_component_x_1() {
        let tokens = vec![
            Atom::from("X1"),
            Atom::from("1"),
            Atom::from("2"),
            Atom::from("3"),
            Atom::from("4"),
            Atom::from("5"),
            Atom::from("6"),
            Atom::from("7"),
            Atom::from("8"),
            Atom::from("9"),
            Atom::from("10"),
            Atom::from("JOHN"),
        ];
        let mut parser = SpiceLineParser::new(&tokens);
        assert_eq!(
            parser.try_parse(),
            Ok(Component::X(XComponent {
                name: Name(Atom::from("X1")),
                pins: vec![
                    Node(Atom::from("1")),
                    Node(Atom::from("2")),
                    Node(Atom::from("3")),
                    Node(Atom::from("4")),
                    Node(Atom::from("5")),
                    Node(Atom::from("6")),
                    Node(Atom::from("7")),
                    Node(Atom::from("8")),
                    Node(Atom::from("9")),
                    Node(Atom::from("10")),
                ],
                sname: Name(Atom::from("JOHN")),
                params: vec![],
                texts: vec![],
            }))
        );
        assert!(parser.is_eof());
    }

    #[test]
    fn test_parse_component_x_2() {
        let tokens = vec![
            Atom::from("X1"),
            Atom::from("1"),
            Atom::from("2"),
            Atom::from("3"),
            Atom::from("4"),
            Atom::from("5"),
            Atom::from("6"),
            Atom::from("7"),
            Atom::from("8"),
            Atom::from("9"),
            Atom::from("10"),
            Atom::from("MICAH"),
            Atom::from("PARAM"),
            Atom::from(":"),
            Atom::from("V1"),
            Atom::from("="),
            Atom::from("2.30"),
            Atom::from("V3"),
            Atom::from("="),
            Atom::from("4.2"),
            Atom::from("TEXT"),
            Atom::from(":"),
            Atom::from("name"),
            Atom::from("="),
            Atom::from("\"Marston\""),
        ];
        let mut parser = SpiceLineParser::new(&tokens);
        assert_eq!(
            parser.try_parse(),
            Ok(Component::X(XComponent {
                name: Name(Atom::from("X1")),
                pins: vec![
                    Node(Atom::from("1")),
                    Node(Atom::from("2")),
                    Node(Atom::from("3")),
                    Node(Atom::from("4")),
                    Node(Atom::from("5")),
                    Node(Atom::from("6")),
                    Node(Atom::from("7")),
                    Node(Atom::from("8")),
                    Node(Atom::from("9")),
                    Node(Atom::from("10")),
                ],
                sname: Name(Atom::from("MICAH")),
                params: vec![
                    Param {
                        name: Name(Atom::from("V1")),
                        value: Number::from(2.30),
                    },
                    Param {
                        name: Name(Atom::from("V3")),
                        value: Number::from(4.2),
                    },
                ],
                texts: vec![Param {
                    name: Name(Atom::from("name")),
                    value: Text(Atom::from("\"Marston\"")),
                }],
            }))
        );
        assert!(parser.is_eof());
    }

    #[test]
    fn test_parse_component_z() {
        let tokens = "ZDRIVE 1 4 2 IGBTA AREA = 10.1u WB = 9u AGD = 4.0u KP = 0.381"
            .split_ascii_whitespace()
            .map(|x| Atom::from(x))
            .collect();
        let mut parser = SpiceLineParser::new(&tokens);
        assert_eq!(
            parser.try_parse(),
            Ok(Component::Z(ZComponent {
                name: Name(Atom::from("ZDRIVE")),
                collector: Node(Atom::from("1")),
                gate: Node(Atom::from("4")),
                emitter: Node(Atom::from("2")),
                model: Name(Atom::from("IGBTA")),
                params: ZComponentParams {
                    area: Some(Number::from(10.1e-6)),
                    wb: Some(Number::from(9e-6)),
                    agd: Some(Number::from(4e-6)),
                    kp: Some(Number::from(0.381)),
                    tau: None,
                }
            }))
        );
        assert!(parser.is_eof());
    }
}

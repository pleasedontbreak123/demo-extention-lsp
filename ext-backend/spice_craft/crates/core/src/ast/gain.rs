use serde::Serialize;
use spice_proc_macro::{Params, TryParse};

use crate::parse::{ParseError, ParseResult, TokenStream, TryParse};

use super::{
    expression::{Expr, Number},
    Name, Node, OptionParenthesis,
};

/// 压控
/// - `<(+) controlling node> <(-) controlling node> <gain>`
///
/// e.g.
/// - `10 11 1.0`
#[derive(Debug, Clone, PartialEq, TryParse, Serialize)]
#[grammar(node1, node2, gain)]
pub struct SimpleVoltageGain {
    pub node1: Node,
    pub node2: Node,
    pub gain: Number,
}

/// 流控
/// - `<controlling V device> <gain>`
///
/// e.g.
/// - `VSENSE 10.0`
#[derive(Debug, Clone, PartialEq, TryParse, Serialize)]
#[grammar(vdevice, gain)]
pub struct SimpleCurrentGain {
    pub vdevice: Name,
    pub gain: Number,
}

/// 压控多项式
/// - `POLY(<value>)`
/// - `+ < <(+) controlling node> <(-) controlling node> >*` (of length `<value>`)
/// - `+ <polynomial coefficient value>*`
///
/// e.g.
/// - `POLY(1) 26 0 0 500`
/// - `POLY(2) 3 0 4 0 0.0 13.6 0.2 0.005`
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PolyVoltageGain {
    pub n: usize,
    pub nodes: Vec<(Node, Node)>,
    pub coefficients: Vec<Number>,
}

impl<T> TryParse<PolyVoltageGain> for T
where
    T: TokenStream,
{
    fn try_parse(&mut self) -> ParseResult<PolyVoltageGain> {
        self.expect("POLY")?;
        self.expect("(")?;
        let n = self.try_parse()?;
        self.expect(")")?;
        let mut nodes = Vec::new();
        for _ in 0..n {
            nodes.push(self.try_parse()?);
        }
        let coefficients = self.try_parse()?;
        Ok(PolyVoltageGain {
            n,
            nodes,
            coefficients,
        })
    }
}

/// 流控多项式
/// - `POLY(<value>)`
/// - `+ <controlling V device name>*` (of length `<value>`)
/// - `+ <polynomial coefficient value>*`
///
/// e.g.
/// - `POLY(1) 26 0 0 500`
/// - `POLY(2) 3 0 4 0 0.0 13.6 0.2 0.005`
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PolyCurrentGain {
    pub n: usize,
    pub vdevices: Vec<Name>,
    pub coefficients: Vec<Number>,
}

impl<T> TryParse<PolyCurrentGain> for T
where
    T: TokenStream,
{
    fn try_parse(&mut self) -> ParseResult<PolyCurrentGain> {
        self.expect("POLY")?;
        self.expect("(")?;
        let n = self.try_parse()?;
        self.expect(")")?;
        let mut vdevices = Vec::new();
        for _ in 0..n {
            vdevices.push(self.try_parse()?);
        }
        let coefficients = self.try_parse()?;
        Ok(PolyCurrentGain {
            n,
            vdevices,
            coefficients,
        })
    }
}

/// 值
/// - `VALUE = { <expression> }`
///
/// e.g.
///
/// - `VALUE = {5V*SQRT(V(3,2))}`
#[derive(Debug, Clone, PartialEq, Serialize, TryParse)]
#[grammar("VALUE", "=", "{", value, "}")]
pub struct ValueGain {
    pub value: Expr,
}

/// 桌子
/// - `TABLE { <expression> } = < ( <input value>, <output value> ) >*`
/// 括号可以省略
#[derive(Debug, Clone, PartialEq, Serialize, TryParse)]
#[grammar("TABLE", "{", expression, "}", "=", values)]
pub struct TableGain {
    pub expression: Expr,
    pub values: Vec<OptionParenthesis<(Number, Number)>>,
}

/// 拉普拉斯
/// - `LAPLACE { <expression> } = { <transform> }`
///
/// e.g.
/// - `LAPLACE {V(10)} = {1/(1+.001*s)}`
#[derive(Debug, Clone, PartialEq, Serialize, TryParse)]
#[grammar("LAPLACE", "{", expression, "}", "=", "{", transform, "}")]
pub struct LaplaceGain {
    pub expression: Expr,
    pub transform: Expr,
}

#[derive(Debug, Clone, PartialEq, TryParse, Serialize)]
#[serde(tag = "type")]
pub enum FreqKeyword {
    /// `MAG` causes magnitude of frequency response to be interpreted as a raw value instead of dB.
    #[matches("MAG")]
    Mag,
    /// `DB` causes magnitude to be interpreted as dB (the default).
    #[matches("DB")]
    Db,
    /// `RAD causes phase to be interpreted in radians.`
    #[matches("RAD")]
    Rad,
    /// `DEG` causes phase to be interpreted in degrees (the default).
    #[matches("DEG")]
    Deg,
    /// `R_I` causes magnitude and phase values to be interpreted as real and imaginary magnitudes.
    #[matches("R_I")]
    Ri,
}

/// 频率
/// - `FREQ { <expression> } = [KEYWORD]`
/// - `+ < ( <frequency value>, <magnitude value>, <phase value> ) >*`
/// - `+ [DELAY = <delay value>]`
/// 括号可以省略
#[derive(Debug, Clone, PartialEq, Serialize, TryParse)]
#[grammar("FREQ", "{", expression, "}", "=", keyword, values, params)]
pub struct FreqGain {
    pub expression: Expr,
    pub keyword: Option<FreqKeyword>,
    pub values: Vec<OptionParenthesis<(Number, Number, Number)>>,
    pub params: FreqGainParams,
}

#[derive(Debug, Clone, PartialEq, Serialize, Params)]
pub struct FreqGainParams {
    #[param(name = "DELAY")]
    pub delay: Option<Number>,
}

/// 切比雪夫
/// - `CHEBYSHEV { <expression> } =`
/// - `+ <[LP] [HP] [BP] [BR]>, <cutoff frequencies>*, <attenuation>*`
///
/// FIXME: 我怎么知道哪里结束 cutoff？（目前是对半分）
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ChebyshevGain {
    pub expression: Expr,
    pub params: ChebyshevGainParams,
    pub cutoffs: Vec<Number>,
    pub attenuations: Vec<Number>,
}

impl<T> TryParse<ChebyshevGain> for T
where
    T: TokenStream,
{
    fn try_parse(&mut self) -> ParseResult<ChebyshevGain> {
        self.expect("CHEBYSHEV")?;
        self.expect("{")?;
        let expression = self.try_parse()?;
        self.expect("}")?;
        self.expect("=")?;
        let params = self.try_parse()?;
        let mut cutoffs: Vec<Number> = self.try_parse()?;
        let attenuations = cutoffs.split_off(cutoffs.len() / 2);
        Ok(ChebyshevGain {
            expression,
            params,
            cutoffs,
            attenuations,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Params)]
pub struct ChebyshevGainParams {
    #[param(name = "LP")]
    pub lp: bool,
    #[param(name = "HP")]
    pub hp: bool,
    #[param(name = "BP")]
    pub bp: bool,
    #[param(name = "BR")]
    pub br: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "type")]
pub enum VoltageGain {
    Simple(SimpleVoltageGain),
    Poly(PolyVoltageGain),
    Value(ValueGain),
    Table(TableGain),
    Laplace(LaplaceGain),
    Freq(FreqGain),
    Chebyshev(ChebyshevGain),
}

impl<T> TryParse<VoltageGain> for T
where
    T: TokenStream,
{
    fn try_parse(&mut self) -> ParseResult<VoltageGain> {
        let token = self.token()?;
        match &token.to_uppercase()[..] {
            "POLY" => Ok(VoltageGain::Poly(self.try_parse()?)),
            "VALUE" => Ok(VoltageGain::Value(self.try_parse()?)),
            "TABLE" => Ok(VoltageGain::Table(self.try_parse()?)),
            "LAPLACE" => Ok(VoltageGain::Laplace(self.try_parse()?)),
            "FREQ" => Ok(VoltageGain::Freq(self.try_parse()?)),
            "CHEBYSHEV" => Ok(VoltageGain::Chebyshev(self.try_parse()?)),
            _ => Ok(VoltageGain::Simple(self.try_parse()?)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "type")]
pub enum CurrentGain {
    Simple(SimpleCurrentGain),
    Poly(PolyCurrentGain),
}

impl<T> TryParse<CurrentGain> for T
where
    T: TokenStream,
{
    fn try_parse(&mut self) -> ParseResult<CurrentGain> {
        let token = self.token()?;
        match &token.to_uppercase()[..] {
            "POLY" => Ok(CurrentGain::Poly(self.try_parse()?)),
            _ => Ok(CurrentGain::Simple(self.try_parse()?)),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        ast::{
            expression::{
                Expr, Factor1, Factor10, Factor11, Factor2, Factor3, Factor4, Factor5, Factor6,
                Factor7, Factor8, Factor9, Ident, Literal,
            },
            Atom,
        },
        parse::SpiceLineParser,
    };

    use super::*;

    #[test]
    fn test_simple_voltage_gain() {
        let tokens = vec![Atom::from("3"), Atom::from("4"), Atom::from("5")];
        let mut parser = SpiceLineParser::new(&tokens);
        assert_eq!(
            parser.try_parse(),
            Ok(VoltageGain::Simple(SimpleVoltageGain {
                node1: Node(Atom::from("3")),
                node2: Node(Atom::from("4")),
                gain: Number::from(5.0),
            }))
        );
        assert!(parser.is_eof());
    }

    #[test]
    fn test_simple_current_gain() {
        let tokens = vec![Atom::from("VSENSE"), Atom::from("10.0")];
        let mut parser = SpiceLineParser::new(&tokens);
        assert_eq!(
            parser.try_parse(),
            Ok(CurrentGain::Simple(SimpleCurrentGain {
                vdevice: Name(Atom::from("VSENSE")),
                gain: Number::from(10.0),
            }))
        );
        assert!(parser.is_eof());
    }

    #[test]
    fn test_poly_voltage_gain() {
        let tokens = vec![
            Atom::from("POLY"),
            Atom::from("("),
            Atom::from("2"),
            Atom::from(")"),
            Atom::from("3"),
            Atom::from("4"),
            Atom::from("5"),
            Atom::from("6"),
            Atom::from("7"),
        ];
        let mut parser = SpiceLineParser::new(&tokens);
        assert_eq!(
            parser.try_parse(),
            Ok(VoltageGain::Poly(PolyVoltageGain {
                n: 2,
                nodes: vec![
                    (Node(Atom::from("3")), Node(Atom::from("4"))),
                    (Node(Atom::from("5")), Node(Atom::from("6"))),
                ],
                coefficients: vec![Number::from(7.0)],
            }))
        );
        assert!(parser.is_eof());
    }

    #[test]
    fn test_poly_current_gain() {
        let tokens = vec![
            Atom::from("POLY"),
            Atom::from("("),
            Atom::from("2"),
            Atom::from(")"),
            Atom::from("VSENSE"),
            Atom::from("VJOHN"),
            Atom::from("10.0"),
            Atom::from("20.0"),
            Atom::from("30.0"),
        ];
        let mut parser = SpiceLineParser::new(&tokens);
        assert_eq!(
            parser.try_parse(),
            Ok(CurrentGain::Poly(PolyCurrentGain {
                n: 2,
                vdevices: vec![Name(Atom::from("VSENSE")), Name(Atom::from("VJOHN"))],
                coefficients: vec![Number::from(10.0), Number::from(20.0), Number::from(30.0)],
            }))
        );
        assert!(parser.is_eof());
    }

    #[test]
    fn test_value_gain() {
        let tokens = vec![
            Atom::from("VALUE"),
            Atom::from("="),
            Atom::from("{"),
            Atom::from("5"),
            Atom::from("}"),
        ];
        let mut parser = SpiceLineParser::new(&tokens);
        assert_eq!(
            parser.try_parse(),
            Ok(VoltageGain::Value(ValueGain {
                value: Expr(Factor11::Base(Factor10::Base(Factor9::Base(
                    Factor8::Base(Factor7::Base(Factor6::Base(Factor5::Base(Factor4::Base(
                        Factor3::Base(Factor2::Base(Factor1::Literal(Literal::from(5))))
                    )))))
                )))),
            }))
        );
        assert!(parser.is_eof());
    }

    #[test]
    fn test_table_gain() {
        let tokens = vec![
            Atom::from("TABLE"),
            Atom::from("{"),
            Atom::from("5"),
            Atom::from("}"),
            Atom::from("="),
            Atom::from("("),
            Atom::from("1"),
            Atom::from("2"),
            Atom::from(")"),
            Atom::from("3"),
            Atom::from("4"),
        ];
        let mut parser = SpiceLineParser::new(&tokens);
        assert_eq!(
            parser.try_parse(),
            Ok(VoltageGain::Table(TableGain {
                expression: Expr(Factor11::Base(Factor10::Base(Factor9::Base(
                    Factor8::Base(Factor7::Base(Factor6::Base(Factor5::Base(Factor4::Base(
                        Factor3::Base(Factor2::Base(Factor1::Literal(Literal::from(5))))
                    )))))
                )))),
                values: vec![
                    OptionParenthesis((Number::from(1.0), Number::from(2.0))),
                    OptionParenthesis((Number::from(3.0), Number::from(4.0)))
                ],
            }))
        );
        assert!(parser.is_eof());
    }

    #[test]
    fn test_laplace_gain() {
        let tokens = "LAPLACE { V ( 10 ) } = { 1 / ( 1 + .001 * s ) }"
            .split_ascii_whitespace()
            .map(|x| Atom::from(x))
            .collect();
        let mut parser = SpiceLineParser::new(&tokens);
        assert_eq!(
            parser.try_parse(),
            Ok(VoltageGain::Laplace(LaplaceGain {
                expression: Expr(Factor11::Base(Factor10::Base(Factor9::Base(
                    Factor8::Base(Factor7::Base(Factor6::Base(Factor5::Base(Factor4::Base(
                        Factor3::Base(Factor2::Base(Factor1::Call {
                            name: Ident::from("V"),
                            args: vec![Expr(Factor11::Base(Factor10::Base(Factor9::Base(
                                Factor8::Base(Factor7::Base(Factor6::Base(Factor5::Base(
                                    Factor4::Base(Factor3::Base(Factor2::Base(Factor1::Literal(
                                        Literal::from(10)
                                    ))))
                                ))))
                            ))))]
                        }))
                    )))))
                )))),
                transform: Expr(Factor11::Base(Factor10::Base(Factor9::Base(
                    Factor8::Base(Factor7::Base(Factor6::Base(Factor5::Base(Factor4::Div(
                        Box::new(Factor4::Base(Factor3::Base(Factor2::Base(
                            Factor1::Literal(Literal::from(1))
                        )))),
                        Box::new(Factor3::Base(Factor2::Base(Factor1::Parenthesis(
                            Box::new(Expr(Factor11::Base(Factor10::Base(Factor9::Base(
                                Factor8::Base(Factor7::Base(Factor6::Base(Factor5::Add(
                                    Box::new(Factor5::Base(Factor4::Base(Factor3::Base(
                                        Factor2::Base(Factor1::Literal(Literal::from(1)))
                                    )))),
                                    Box::new(Factor4::Mul(
                                        Box::new(Factor4::Base(Factor3::Base(Factor2::Base(
                                            Factor1::Literal(Literal::from(0.001))
                                        )))),
                                        Box::new(Factor3::Base(Factor2::Base(Factor1::Ident(
                                            Ident::from("s")
                                        ))))
                                    ))
                                ))))
                            )))))
                        ))))
                    )))))
                )))),
            }))
        );
        assert!(parser.is_eof());
    }

    #[test]
    fn test_freq_gain() {
        let tokens = vec![
            Atom::from("FREQ"),
            Atom::from("{"),
            Atom::from("V"),
            Atom::from("("),
            Atom::from("10"),
            Atom::from(")"),
            Atom::from("}"),
            Atom::from("="),
            Atom::from("MAG"),
            Atom::from("("),
            Atom::from("1"),
            Atom::from("2"),
            Atom::from("3"),
            Atom::from(")"),
            Atom::from("4"),
            Atom::from("5"),
            Atom::from("6"),
            Atom::from("("),
            Atom::from("7"),
            Atom::from("8"),
            Atom::from("9"),
            Atom::from(")"),
            Atom::from("DELAY"),
            Atom::from("="),
            Atom::from("16"),
        ];
        let mut parser = SpiceLineParser::new(&tokens);
        assert_eq!(
            parser.try_parse(),
            Ok(VoltageGain::Freq(FreqGain {
                expression: Expr(Factor11::Base(Factor10::Base(Factor9::Base(
                    Factor8::Base(Factor7::Base(Factor6::Base(Factor5::Base(Factor4::Base(
                        Factor3::Base(Factor2::Base(Factor1::Call {
                            name: Ident::from("V"),
                            args: vec![Expr(Factor11::Base(Factor10::Base(Factor9::Base(
                                Factor8::Base(Factor7::Base(Factor6::Base(Factor5::Base(
                                    Factor4::Base(Factor3::Base(Factor2::Base(Factor1::Literal(
                                        Literal::from(10)
                                    ))))
                                ))))
                            ))))]
                        }))
                    )))))
                )))),
                keyword: Some(FreqKeyword::Mag),
                values: vec![
                    OptionParenthesis((Number::from(1.0), Number::from(2.0), Number::from(3.0))),
                    OptionParenthesis((Number::from(4.0), Number::from(5.0), Number::from(6.0))),
                    OptionParenthesis((Number::from(7.0), Number::from(8.0), Number::from(9.0))),
                ],
                params: FreqGainParams {
                    delay: Some(Number::from(16.0)),
                }
            }))
        );
        assert!(parser.is_eof());
    }

    #[test]
    fn test_freq_keyword() {
        let tokens = vec![
            Atom::from("MAG"),
            Atom::from("DB"),
            Atom::from("RAD"),
            Atom::from("DEG"),
            Atom::from("R_I"),
        ];
        let mut parser = SpiceLineParser::new(&tokens);
        assert_eq!(parser.try_parse(), Ok(FreqKeyword::Mag));
        assert_eq!(parser.try_parse(), Ok(FreqKeyword::Db));
        assert_eq!(parser.try_parse(), Ok(FreqKeyword::Rad));
        assert_eq!(parser.try_parse(), Ok(FreqKeyword::Deg));
        assert_eq!(parser.try_parse(), Ok(FreqKeyword::Ri));
        assert!(parser.is_eof());
    }

    #[test]
    fn test_chebyshev_gain() {
        let tokens = vec![
            Atom::from("CHEBYSHEV"),
            Atom::from("{"),
            Atom::from("5"),
            Atom::from("}"),
            Atom::from("="),
            Atom::from("LP"),
            Atom::from("HP"),
            Atom::from("BP"),
            Atom::from("BR"),
            Atom::from("1"),
            Atom::from("2"),
            Atom::from("3"),
            Atom::from("4"),
            Atom::from("5"),
            Atom::from("6"),
        ];
        let mut parser = SpiceLineParser::new(&tokens);
        assert_eq!(
            parser.try_parse(),
            Ok(VoltageGain::Chebyshev(ChebyshevGain {
                expression: Expr(Factor11::Base(Factor10::Base(Factor9::Base(
                    Factor8::Base(Factor7::Base(Factor6::Base(Factor5::Base(Factor4::Base(
                        Factor3::Base(Factor2::Base(Factor1::Literal(Literal::from(5))))
                    )))))
                )))),
                params: ChebyshevGainParams {
                    lp: true,
                    hp: true,
                    bp: true,
                    br: true,
                },
                cutoffs: vec![Number::from(1.0), Number::from(2.0), Number::from(3.0)],
                attenuations: vec![Number::from(4.0), Number::from(5.0), Number::from(6.0)],
            }))
        );
        assert!(parser.is_eof());
    }
}

use serde::Serialize;
use spice_proc_macro::TryParse;

use crate::parse::{ParseError, ParseResult, TokenStream, TryParse};

use super::expression::Number;

/// 直流分量
/// `[DC] <value>`
#[derive(Debug, Clone, PartialEq, Serialize, TryParse)]
#[grammar(dc("DC"), value)]
pub struct DcSource {
    pub dc: bool,
    pub value: Number,
}

/// 交流分量
/// `AC <magnitude> [<phase>]`
#[derive(Debug, Clone, PartialEq, TryParse, Serialize)]
#[grammar("AC", magnitude, phase)]
pub struct AcSource {
    pub magnitude: Number,
    pub phase: Option<Number>,
}

/// 脉冲
///
/// `PULSE ( <v1> <v2> [<td>] [<tr>] [<tf>] [<pw>] [<per>] )`
#[derive(Debug, Clone, PartialEq, TryParse, Serialize)]
#[grammar("PULSE", "(", v1, v2, td, tr, tf, pw, per, ")")]
pub struct Pulse {
    pub v1: Number,
    pub v2: Number,
    /// defaults to 0
    pub td: Option<Number>,
    /// defaults to TSTEP
    pub tr: Option<Number>,
    /// defaults to TSTEP
    pub tf: Option<Number>,
    /// defaults to TSTOP
    pub pw: Option<Number>,
    /// defaults to TSTOP
    pub per: Option<Number>,
}

/// 分段线性
///
/// - `PWL ( <t1> <v1> <t2> <v2> ... )`
#[derive(Debug, Clone, PartialEq, TryParse, Serialize)]
#[grammar("PWL", "(", points, ")")]
pub struct Pwl {
    pub points: Vec<(Number, Number)>,
}

/// 正弦
///
/// - `SIN ( <v0> <vampl> [<freq>] [<td>] [<alpha>] [<theta>] )`
#[derive(Debug, Clone, PartialEq, TryParse, Serialize)]
#[grammar("SIN", "(", v0, vampl, freq, td, alpha, theta, ")")]
pub struct Sin {
    pub v0: Number,
    pub vampl: Number,
    /// defaults to 1/TSTOP
    pub freq: Option<Number>,
    /// defaults to 0
    pub td: Option<Number>,
    /// defaults to 0
    pub alpha: Option<Number>,
    /// defaults to 0
    pub theta: Option<Number>,
}

/// 指数
///
/// - `EXP ( <v1> <v2> [<td1>] [<tc1>] [<td2>] [<tc2>] )`
#[derive(Debug, Clone, PartialEq, TryParse, Serialize)]
#[grammar("EXP", "(", v1, v2, td1, tc1, td2, tc2, ")")]
pub struct Exp {
    pub v1: Number,
    pub v2: Number,
    /// defaults to 0
    pub td1: Option<Number>,
    /// defaults to TSTEP
    pub tc1: Option<Number>,
    /// defaults to TD1 + TSTEP
    pub td2: Option<Number>,
    /// defaults to TSTEP
    pub tc2: Option<Number>,
}

/// 单频率调频
///
/// - `SFFM ( <v0> <va> [<fc>] [<md>] [<fs>] )`
#[derive(Debug, Clone, PartialEq, TryParse, Serialize)]
#[grammar("SFFM", "(", v0, va, fc, md, fs, ")")]
pub struct Sffm {
    pub v0: Number,
    pub va: Number,
    /// defaults to 1/TSTOP
    pub fc: Option<Number>,
    /// defaults to 0
    pub md: Option<Number>,
    /// defaults to 1/TSTOP
    pub fs: Option<Number>,
}

/// 交流
///
/// - `VAC ( <dc> <ac_mag> <ac_phase> )`
#[derive(Debug, Clone, PartialEq, TryParse, Serialize)]
#[grammar("VAC", "(", dc, ac_mag, ac_phase, ")")]
pub struct Vac {
    pub dc: Number,
    pub ac_mag: Number,
    pub ac_phase: Number,
}

/// 信号源
///
/// `<NAME> ( <v1> <v2> ... <vn> )`
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "type")]
pub enum Transient {
    Pulse(Pulse),
    Pwl(Pwl),
    Sin(Sin),
    Exp(Exp),
    Sffm(Sffm),
    Vac(Vac),
}

impl<T> TryParse<Transient> for T
where
    T: TokenStream,
{
    fn try_parse(&mut self) -> ParseResult<Transient> {
        let token = self.token()?;
        match &token.to_uppercase()[..] {
            "PULSE" => Ok(Transient::Pulse(self.try_parse()?)),
            "PWL" => Ok(Transient::Pwl(self.try_parse()?)),
            "SIN" => Ok(Transient::Sin(self.try_parse()?)),
            "EXP" => Ok(Transient::Exp(self.try_parse()?)),
            "SFFM" => Ok(Transient::Sffm(self.try_parse()?)),
            "VAC" => Ok(Transient::Vac(self.try_parse()?)),
            _ => Err(ParseError {
                reason: format!("Expect `{}' to be a transient specification", token.raw),
                position: Some(token.column),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{spice_test_err, spice_test_ok};

    use super::*;

    #[test]
    fn test_parse_dc() {
        spice_test_ok!(
            "DC 2",
            DcSource {
                dc: true,
                value: Number::from(2.0)
            }
        );
        spice_test_ok!(
            "2",
            DcSource {
                dc: false,
                value: Number::from(2.0)
            }
        );
    }

    #[test]
    fn test_parse_transient_pluse() {
        spice_test_ok!(
            "PULSE ( 1 2 3 4 5 6 7 )",
            Pulse {
                v1: Number::from(1.0),
                v2: Number::from(2.0),
                td: Some(Number::from(3.0)),
                tr: Some(Number::from(4.0)),
                tf: Some(Number::from(5.0)),
                pw: Some(Number::from(6.0)),
                per: Some(Number::from(7.0)),
            }
        );
    }

    #[test]
    fn test_parse_transient_pwl() {
        spice_test_ok!(
            "PWL ( 0 1 2 3 )",
            Pwl {
                points: vec![
                    (Number::from(0.0), Number::from(1.0)),
                    (Number::from(2.0), Number::from(3.0)),
                ],
            }
        );
        spice_test_err!("PWL ( 2 )", Pwl);
        spice_test_err!("PWL ( 2 3", Pwl);
    }

    #[test]
    fn test_parse_transient_sin() {
        spice_test_ok!(
            "SIN ( 1 2 3 4 5 6 )",
            Sin {
                v0: Number::from(1.0),
                vampl: Number::from(2.0),
                freq: Some(Number::from(3.0)),
                td: Some(Number::from(4.0)),
                alpha: Some(Number::from(5.0)),
                theta: Some(Number::from(6.0)),
            }
        );
        spice_test_err!("SIN ( 1 )", Sin);
    }

    #[test]
    fn test_parse_transient_exp() {
        spice_test_ok!(
            "EXP ( 1 2 3 4 5 6 )",
            Exp {
                v1: Number::from(1.0),
                v2: Number::from(2.0),
                td1: Some(Number::from(3.0)),
                tc1: Some(Number::from(4.0)),
                td2: Some(Number::from(5.0)),
                tc2: Some(Number::from(6.0)),
            }
        );
        spice_test_err!("EXP ( 2 )", Exp);
    }

    #[test]
    fn test_parse_transient_sffm() {
        spice_test_ok!(
            "SFFM ( 1 2 3 4 5 )",
            Sffm {
                v0: Number::from(1.0),
                va: Number::from(2.0),
                fc: Some(Number::from(3.0)),
                md: Some(Number::from(4.0)),
                fs: Some(Number::from(5.0)),
            }
        );
        spice_test_err!("SFFM ( 2 )", Sffm);
    }

    #[test]
    fn test_parse_transient_vac() {
        spice_test_ok!(
            "VAC ( 1 2 3 )",
            Vac {
                dc: Number::from(1.0),
                ac_mag: Number::from(2.0),
                ac_phase: Number::from(3.0),
            }
        );
    }

    #[test]
    fn test_parse_transient_sin_less() {
        spice_test_ok!(
            "SIN ( 1 2 3 )",
            Sin {
                v0: Number::from(1.0),
                vampl: Number::from(2.0),
                freq: Some(Number::from(3.0)),
                td: None,
                alpha: None,
                theta: None,
            }
        );
    }
}

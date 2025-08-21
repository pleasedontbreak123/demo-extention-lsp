use serde::Serialize;
use spice_proc_macro::TryParse;

use crate::parse::{ParseResult, TokenStream, TryParse};

use super::{expression::Number, Name};

/// 扫描的目标
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "type")]
pub enum SweepName {
    /// 电源，可以是 独立电压源 或者 独立电流源（V 开头或者 I 开头）
    /// - `V2`
    /// - `I3`
    /// - `<name>`
    Power { name: Name },
    /// 模型参数
    /// - `<name1> <name2> ( <param> )`
    ModelParam {
        /// 模型名字类型
        name1: Name,
        /// 模型名字
        name2: Name,
        /// 模型参数名字
        param: Name,
    },
    /// 温度
    ///
    /// TODO: 只支持 LIST
    ///
    /// - `TEMP`
    Temp,
    /// 全局变量
    /// - `PARAM <name>`
    Param { name: Name },
}

impl<T> TryParse<SweepName> for T
where
    T: TokenStream,
{
    fn try_parse(&mut self) -> ParseResult<SweepName> {
        let token = self.token()?;
        match &token.to_uppercase()[..] {
            "TEMP" => {
                self.consume();
                Ok(SweepName::Temp)
            }
            "PARAM" => {
                self.consume();
                let name = self.try_parse()?;
                Ok(SweepName::Param { name })
            }
            name if name.starts_with("V") || name.starts_with("I") => {
                let name = self.try_parse()?;
                Ok(SweepName::Power { name })
            }
            _ => {
                let name1 = self.try_parse()?;
                let name2 = self.try_parse()?;
                self.expect("(")?;
                let param = self.try_parse()?;
                self.expect(")")?;
                Ok(SweepName::ModelParam {
                    name1,
                    name2,
                    param,
                })
            }
        }
    }
}

/// Linear 线性扫描
/// - `<swname> <sstart> <send> <sinc>`
/// - `LIN <swname> <sstart> <send> <sinc>`
#[derive(Debug, Clone, PartialEq, TryParse, Serialize)]
#[grammar("LIN", swname, sstart, send, sinc)]
pub struct Lin {
    pub swname: SweepName,
    pub sstart: Number,
    pub send: Number,
    pub sinc: Number,
}

/// 八倍扫描
/// - `OCT <swname> <sstart> <send> <np>`
#[derive(Debug, Clone, PartialEq, TryParse, Serialize)]
#[grammar("OCT", swname, sstart, send, np)]
pub struct Oct {
    pub swname: SweepName,
    pub sstart: Number,
    pub send: Number,
    pub np: Number,
}

/// 十倍扫描
/// - `DEC <swname> <sstart> <send> <np>`
#[derive(Debug, Clone, PartialEq, TryParse, Serialize)]
#[grammar("DEC", swname, sstart, send, np)]
pub struct Dec {
    pub swname: SweepName,
    pub sstart: Number,
    pub send: Number,
    pub np: Number,
}

/// 列表枚举
/// - `<swname> LIST <v1> <v2> ...`
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct List {
    pub swname: SweepName,
    pub values: Vec<Number>,
}

/// 扫描类型
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "type")]
pub enum Sweep {
    Lin(Lin),
    Oct(Oct),
    Dec(Dec),
    List(List),
}

impl<T> TryParse<Sweep> for T
where
    T: TokenStream,
{
    fn try_parse(&mut self) -> ParseResult<Sweep> {
        let token = self.token()?;
        match &token.to_uppercase()[..] {
            "OCT" => Ok(Sweep::Oct(self.try_parse()?)),
            "DEC" => Ok(Sweep::Dec(self.try_parse()?)),
            "LIN" => Ok(Sweep::Lin(self.try_parse()?)),
            _ => {
                let swname = self.try_parse()?;
                if self.matches_consume("LIST") {
                    let values = self.try_parse()?;
                    Ok(Sweep::List(List { swname, values }))
                } else {
                    let sstart = self.try_parse()?;
                    let send = self.try_parse()?;
                    let sinc = self.try_parse()?;
                    Ok(Sweep::Lin(Lin {
                        swname,
                        sstart,
                        send,
                        sinc,
                    }))
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::spice_test_ok;

    use super::*;

    #[test]
    fn test_parse_sweep_lin() {
        spice_test_ok!(
            "LIN TEMP 0 100 10",
            Sweep::Lin(Lin {
                swname: SweepName::Temp,
                sstart: Number::from(0.0),
                send: Number::from(100.0),
                sinc: Number::from(10.0),
            })
        );
    }

    #[test]
    fn test_parse_sweep_oct() {
        spice_test_ok!(
            "OCT TEMP 0 100 10",
            Sweep::Oct(Oct {
                swname: SweepName::Temp,
                sstart: Number::from(0.0),
                send: Number::from(100.0),
                np: Number::from(10.0),
            })
        );
    }

    #[test]
    fn test_parse_sweep_dec() {
        spice_test_ok!(
            "DEC TEMP 0 100 10",
            Sweep::Dec(Dec {
                swname: SweepName::Temp,
                sstart: Number::from(0.0),
                send: Number::from(100.0),
                np: Number::from(10.0),
            })
        );
    }

    #[test]
    fn test_parse_sweep_temp_lin() {
        spice_test_ok!(
            "TEMP 0 100 10",
            Sweep::Lin(Lin {
                swname: SweepName::Temp,
                sstart: Number::from(0.0),
                send: Number::from(100.0),
                sinc: Number::from(10.0),
            })
        );
    }

    #[test]
    fn test_parse_sweep_temp_list() {
        spice_test_ok!(
            "TEMP LIST 100 10 20 30 40",
            Sweep::List(List {
                swname: SweepName::Temp,
                values: vec![
                    Number::from(100.0),
                    Number::from(10.0),
                    Number::from(20.0),
                    Number::from(30.0),
                    Number::from(40.0),
                ],
            })
        );
    }
}

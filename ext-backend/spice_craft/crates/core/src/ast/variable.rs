use crate::{
    parse::{ParseError, ParseResult, TokenStream},
    TryParse,
};
use serde::Serialize;

use super::{ExplicitNode, Name};

pub const TWO_TERMINAL_COMPONENTS: &str = "CDEFGHILRSVW";
pub const TRHEE_OR_FOUR_TERMINAL_COMPONENTS: &str = "BJMQZ";
pub const TRANSMISSION_COMPONENTS: &str = "T";

/// - `V(2)`
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "type")]
pub enum VoltageVariable {
    /// - `V(2)`
    /// - `V([node])`
    Node { node: ExplicitNode, suffix: Suffix },
    /// - `V(1,2)`
    /// - `V([vcc],[gnd])`
    Node2 {
        node1: ExplicitNode,
        node2: ExplicitNode,
        suffix: Suffix,
    },
    /// Name must be in `CDEFGHILRSVW` (with 2 terminals).
    /// - `V(E1)`
    Name { name: Name, suffix: Suffix },
    /// Name must be in `BJMQZ` (with 3-4 terminals)
    /// - `VG(J2)`
    NameX {
        name: Name,
        x: Terminal,
        suffix: Suffix,
    },
    /// Name must be in `T` (transmission line)
    /// - `VA(T1)`
    /// - `VA(T2)`
    NameZ {
        name: Name,
        z: TransmissionEnd,
        suffix: Suffix,
    },
    /// Name must be in `BJMQZ` (with 3-4 terminals)
    /// - `VGB(J3)`
    NameXY {
        name: Name,
        x: Terminal,
        y: Terminal,
        suffix: Suffix,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum Terminal {
    /// `B` 基极
    B,
    /// `C` 集电极
    C,
    /// `D` 漏极
    D,
    /// `E` 发射极
    E,
    /// `G` 栅极
    G,
    /// `S` 原极
    S,
}

impl Terminal {
    fn try_from_str(s: &str) -> Option<Terminal> {
        match s {
            "B" => Some(Terminal::B),
            "C" => Some(Terminal::C),
            "D" => Some(Terminal::D),
            "E" => Some(Terminal::E),
            "G" => Some(Terminal::G),
            "S" => Some(Terminal::S),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum TransmissionEnd {
    A,
    B,
}

impl TransmissionEnd {
    fn try_from_str(s: &str) -> Option<TransmissionEnd> {
        match s {
            "A" => Some(TransmissionEnd::A),
            "B" => Some(TransmissionEnd::B),
            _ => None,
        }
    }
}

const ACCEPT_SUFFIXES: &str = "<None>, `DB', `G', `I', `M', `P' or `R'";

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum Suffix {
    /// magnitude
    None,
    /// magnitude in decibels
    Db,
    /// group delay (-dPHASE/dFREQUENCY)
    G,
    /// imaginary part
    I,
    /// magnitude
    M,
    /// phase in degrees
    P,
    /// real part
    R,
}

impl Suffix {
    fn try_from_str(s: &str) -> Option<Suffix> {
        match s {
            "" => Some(Suffix::None),
            "DB" => Some(Suffix::Db),
            "G" => Some(Suffix::G),
            "I" => Some(Suffix::I),
            "M" => Some(Suffix::M),
            "P" => Some(Suffix::P),
            "R" => Some(Suffix::R),
            _ => None,
        }
    }
}

/// - `I(R1)`
#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum CurrentVariable {
    /// Name must be in `CDEFGHILRSVW` (with 2 terminals).
    /// - `I(E2)`
    Name { name: Name, suffix: Suffix },
    /// Name must be in `BJMQZ` (with 3-4 terminals)
    /// - `IG(J2)`
    NameX {
        name: Name,
        x: Terminal,
        suffix: Suffix,
    },
    /// Name must be in `T` (transmission line)
    /// - `Ia(T1)`
    /// - `Ib(T2)`
    NameZ {
        name: Name,
        z: TransmissionEnd,
        suffix: Suffix,
    },
}

/// - `W(R1)`
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PowerVariable {
    name: Name,
    suffix: Suffix,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum NoiseVariable {
    /// `ONOISE`
    Onoise,
    /// `INOISE`
    Inoise,
    /// `DB(ONOISE)`
    DbOnoise,
    /// `DB(INOISE)`
    DbInoise,
}

/// 输出变量（AC 分析用）
///
/// 参考书 3.2 节 / 68 页
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "type", content = "value")]
pub enum OutputVariable {
    /// 电压值
    /// - `V(...)`
    Voltage(VoltageVariable),

    /// 电流值
    /// - `I(...)`
    Current(CurrentVariable),

    /// 功率
    /// - `W(...)`
    Power(PowerVariable),

    /// 噪声
    /// - `INOISE`
    Noise(NoiseVariable),
}

/// Note: 目前是照着英语书做的
impl<T> TryParse<OutputVariable> for T
where
    T: TokenStream,
{
    fn try_parse(&mut self) -> ParseResult<OutputVariable> {
        let token = self.token()?;
        match &token.to_uppercase()[..] {
            "INOISE" => {
                self.consume();
                Ok(OutputVariable::Noise(NoiseVariable::Inoise))
            }
            "ONOISE" => {
                self.consume();
                Ok(OutputVariable::Noise(NoiseVariable::Onoise))
            }
            "DB" => {
                self.consume();
                self.expect("(")?;
                let token = self.token()?;
                match &token.to_uppercase()[..] {
                    "INOISE" => {
                        self.consume();
                        self.expect(")")?;
                        Ok(OutputVariable::Noise(NoiseVariable::DbInoise))
                    }
                    "ONOISE" => {
                        self.consume();
                        self.expect(")")?;
                        Ok(OutputVariable::Noise(NoiseVariable::DbOnoise))
                    }
                    _ => Err(ParseError::unexpected(
                        "INOISE' or `ONOISE",
                        &token.raw[..],
                        Some(token.column),
                    )),
                }
            }

            name if name.starts_with("W") => {
                let suffix = &name[1..];

                self.consume();
                self.expect("(")?;
                let name = self.try_parse()?;
                self.expect(")")?;

                let suffix = Suffix::try_from_str(suffix).ok_or_else(|| {
                    ParseError::unknown_suffix(suffix, ACCEPT_SUFFIXES, Some(token.column))
                })?;

                Ok(OutputVariable::Power(PowerVariable { name, suffix }))
            }

            name if name.starts_with("V") => {
                let mut suffix = &name[1..];
                self.consume();
                self.expect("(")?;

                if let Some(node1) = self.try_parse()? {
                    if let Some(node2) = self.try_parse()? {
                        self.expect(")")?;
                        let suffix = Suffix::try_from_str(suffix).ok_or_else(|| {
                            ParseError::unknown_suffix(suffix, ACCEPT_SUFFIXES, Some(token.column))
                        })?;
                        return Ok(OutputVariable::Voltage(VoltageVariable::Node2 {
                            node1,
                            node2,
                            suffix,
                        }));
                    }
                    self.expect(")")?;
                    let suffix = Suffix::try_from_str(suffix).ok_or_else(|| {
                        ParseError::unknown_suffix(suffix, ACCEPT_SUFFIXES, Some(token.column))
                    })?;
                    return Ok(OutputVariable::Voltage(VoltageVariable::Node {
                        node: node1,
                        suffix,
                    }));
                }

                let name: Name = self.try_parse()?;
                self.expect(")")?;

                let name_type = name
                    .0
                    .raw
                    .chars()
                    .next()
                    .map(|x| x.to_ascii_uppercase())
                    .unwrap();

                if TWO_TERMINAL_COMPONENTS.contains(name_type) {
                    let suffix = Suffix::try_from_str(suffix).ok_or_else(|| {
                        ParseError::unknown_suffix(suffix, ACCEPT_SUFFIXES, Some(token.column))
                    })?;
                    return Ok(OutputVariable::Voltage(VoltageVariable::Name {
                        name,
                        suffix,
                    }));
                }

                if TRHEE_OR_FOUR_TERMINAL_COMPONENTS.contains(name_type) {
                    if let Some(pin1) = suffix.get(..1).and_then(|x| Terminal::try_from_str(x)) {
                        suffix = &suffix[1..];

                        if let Some(pin2) = suffix.get(..1).and_then(|x| Terminal::try_from_str(x))
                        {
                            suffix = &suffix[1..];

                            let suffix = Suffix::try_from_str(suffix).ok_or_else(|| {
                                ParseError::unknown_suffix(
                                    suffix,
                                    ACCEPT_SUFFIXES,
                                    Some(token.column),
                                )
                            })?;

                            return Ok(OutputVariable::Voltage(VoltageVariable::NameXY {
                                name,
                                x: pin1,
                                y: pin2,
                                suffix,
                            }));
                        }

                        let suffix = Suffix::try_from_str(suffix).ok_or_else(|| {
                            ParseError::unknown_suffix(suffix, ACCEPT_SUFFIXES, Some(token.column))
                        })?;

                        return Ok(OutputVariable::Voltage(VoltageVariable::NameX {
                            name,
                            x: pin1,
                            suffix,
                        }));
                    }

                    return Err(ParseError {
                        reason: format!("Voltage of Element must provide at least one terminal"),
                        position: Some(token.column),
                    });
                }

                if TRANSMISSION_COMPONENTS.contains(name_type) {
                    if let Some(pin1) = suffix
                        .get(..1)
                        .and_then(|x| TransmissionEnd::try_from_str(x))
                    {
                        suffix = &suffix[1..];
                        let suffix = Suffix::try_from_str(suffix).ok_or_else(|| {
                            ParseError::unknown_suffix(suffix, ACCEPT_SUFFIXES, Some(token.column))
                        })?;

                        return Ok(OutputVariable::Voltage(VoltageVariable::NameZ {
                            name,
                            z: pin1,
                            suffix,
                        }));
                    }

                    return Err(ParseError {
                        reason: format!("Voltage of Transmission Line must provide A or B end"),
                        position: Some(token.column),
                    });
                }

                return Err(ParseError {
                    reason: format!("Unsupported element type: `{}'", name_type),
                    position: Some(token.column),
                });
            }

            name if name.starts_with("I") => {
                let mut suffix = &name[1..];
                self.consume();

                self.expect("(")?;
                let name: Name = self.try_parse()?;
                self.expect(")")?;

                let name_type = name
                    .0
                    .raw
                    .chars()
                    .next()
                    .map(|x| x.to_ascii_uppercase())
                    .unwrap();

                if TWO_TERMINAL_COMPONENTS.contains(name_type) {
                    let suffix = Suffix::try_from_str(suffix).ok_or_else(|| {
                        ParseError::unknown_suffix(suffix, ACCEPT_SUFFIXES, Some(token.column))
                    })?;
                    return Ok(OutputVariable::Current(CurrentVariable::Name {
                        name,
                        suffix,
                    }));
                }

                if TRHEE_OR_FOUR_TERMINAL_COMPONENTS.contains(name_type) {
                    if let Some(pin1) = suffix.get(..1).and_then(|x| Terminal::try_from_str(x)) {
                        suffix = &suffix[1..];

                        let suffix = Suffix::try_from_str(suffix).ok_or_else(|| {
                            ParseError::unknown_suffix(suffix, ACCEPT_SUFFIXES, Some(token.column))
                        })?;
                        return Ok(OutputVariable::Current(CurrentVariable::NameX {
                            name,
                            x: pin1,
                            suffix,
                        }));
                    }

                    return Err(ParseError {
                        reason: format!("Current of Element must provide at least one terminal"),
                        position: Some(token.column),
                    });
                }

                if TRANSMISSION_COMPONENTS.contains(name_type) {
                    if let Some(pin1) = suffix
                        .get(..1)
                        .and_then(|x| TransmissionEnd::try_from_str(x))
                    {
                        suffix = &suffix[1..];

                        let suffix = Suffix::try_from_str(suffix).ok_or_else(|| {
                            ParseError::unknown_suffix(suffix, ACCEPT_SUFFIXES, Some(token.column))
                        })?;
                        return Ok(OutputVariable::Current(CurrentVariable::NameZ {
                            name,
                            z: pin1,
                            suffix,
                        }));
                    }

                    return Err(ParseError {
                        reason: format!("Current of Transmission Line must provide A or B end"),
                        position: Some(token.column),
                    });
                }

                return Err(ParseError {
                    reason: format!("Unsupported element type: `{}'", name_type),
                    position: Some(token.column),
                });
            }

            _ => Err(ParseError {
                reason: format!("Expect `{}' to be an output variable", token.raw),
                position: Some(token.column),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        ast::{
            variable::{
                CurrentVariable, NoiseVariable, OutputVariable, PowerVariable, Suffix, Terminal,
                TransmissionEnd, VoltageVariable,
            },
            Atom, ExplicitNode, Name, Node,
        },
        parse::TokenStream,
        spice_test_ok, TryParse,
    };

    #[test]
    fn test_parse_output_variable() {
        spice_test_ok!("INOISE", OutputVariable::Noise(NoiseVariable::Inoise));
        spice_test_ok!("ONOISE", OutputVariable::Noise(NoiseVariable::Onoise));
        spice_test_ok!("DB(INOISE)", OutputVariable::Noise(NoiseVariable::DbInoise));
        spice_test_ok!("DB(ONOISE)", OutputVariable::Noise(NoiseVariable::DbOnoise));

        spice_test_ok!(
            "V(10)",
            OutputVariable::Voltage(VoltageVariable::Node {
                node: ExplicitNode(Node(Atom::from("10"))),
                suffix: Suffix::None
            })
        );

        spice_test_ok!(
            "II(R13)",
            OutputVariable::Current(CurrentVariable::Name {
                name: Name(Atom::from("R13")),
                suffix: Suffix::I
            })
        );
        spice_test_ok!(
            "IGG(M3)",
            OutputVariable::Current(CurrentVariable::NameX {
                name: Name(Atom::from("M3")),
                x: Terminal::G,
                suffix: Suffix::G
            })
        );
        spice_test_ok!(
            "IR(VIN)",
            OutputVariable::Current(CurrentVariable::Name {
                name: Name(Atom::from("VIN")),
                suffix: Suffix::R
            })
        );
        spice_test_ok!(
            "IAG(T2)",
            OutputVariable::Current(CurrentVariable::NameZ {
                name: Name(Atom::from("T2")),
                z: TransmissionEnd::A,
                suffix: Suffix::G
            })
        );
        spice_test_ok!(
            "V(2,3)",
            OutputVariable::Voltage(VoltageVariable::Node2 {
                node1: ExplicitNode(Node(Atom::from("2"))),
                node2: ExplicitNode(Node(Atom::from("3"))),
                suffix: Suffix::None,
            })
        );
        spice_test_ok!(
            "VDB(R1)",
            OutputVariable::Voltage(VoltageVariable::Name {
                name: Name(Atom::from("R1")),
                suffix: Suffix::Db
            })
        );
        spice_test_ok!(
            "VBEP(Q3)",
            OutputVariable::Voltage(VoltageVariable::NameXY {
                name: Name(Atom::from("Q3")),
                x: Terminal::B,
                y: Terminal::E,
                suffix: Suffix::P
            })
        );
        spice_test_ok!(
            "VM(2)",
            OutputVariable::Voltage(VoltageVariable::Node {
                node: ExplicitNode(Node(Atom::from("2"))),
                suffix: Suffix::M
            })
        );
        spice_test_ok!(
            "WM(U7)",
            OutputVariable::Power(PowerVariable {
                name: Name(Atom::from("U7")),
                suffix: Suffix::M,
            })
        );
    }
}

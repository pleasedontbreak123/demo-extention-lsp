use anyhow::anyhow;
use nom::{
    branch::alt,
    bytes::complete::tag,
    combinator::{map, value},
    number::complete::double,
    sequence::tuple,
    Parser,
};
pub trait NumberParser {
    fn to_number(&self, a: &str) -> anyhow::Result<f64>;
}

pub struct SuffixNumberParser;

impl NumberParser for SuffixNumberParser {
    /// This function parses a number, with optionally suffix.
    ///
    /// TODO: LT mode: RKM notation, e.g. 2k7 or 100r.
    fn to_number(&self, n: &str) -> anyhow::Result<f64> {
        let n = n.to_ascii_lowercase();
        let number = alt((parse_suffix_number(), double))
            .parse(&n)
            .map_err(|x| anyhow!("{x}"))?
            .1;
        Ok(number)
    }
}

#[derive(Debug, Clone)]
///
pub enum Suffix {
    Tera,
    Giga,
    /// `meg` suffix in spice.
    Mega,
    Kilo,
    Mil,
    /// `m` suffix in spice.
    Milli,
    Micro,
    Nano,
    Pico,
    Femto,
    Atto,
}

impl Suffix {
    pub fn factor(&self) -> f64 {
        match self {
            Suffix::Tera => 1e12,
            Suffix::Giga => 1e9,
            Suffix::Mega => 1e6,
            Suffix::Kilo => 1e3,
            Suffix::Mil => 25.4e-6,
            Suffix::Milli => 1e-3,
            Suffix::Micro => 1e-6,
            Suffix::Nano => 1e-9,
            Suffix::Pico => 1e-12,
            Suffix::Femto => 1e-15,
            Suffix::Atto => 1e-18,
        }
    }

    pub fn value(&self, coeff: f64) -> f64 {
        self.factor() * coeff
    }
}

fn parse_suffix<'a>() -> impl Parser<&'a str, Suffix, nom::error::Error<&'a str>> {
    alt((
        value(Suffix::Tera, tag("t")),
        value(Suffix::Giga, tag("g")),
        value(Suffix::Mega, tag("meg")),
        value(Suffix::Kilo, tag("k")),
        value(Suffix::Mil, tag("mil")),
        value(Suffix::Milli, tag("m")),
        value(Suffix::Micro, tag("u")),
        value(Suffix::Nano, tag("n")),
        value(Suffix::Pico, tag("p")),
        value(Suffix::Femto, tag("f")),
        value(Suffix::Atto, tag("a")),
    ))
}

fn parse_suffix_number<'a>() -> impl Parser<&'a str, f64, nom::error::Error<&'a str>> {
    map(tuple((double, parse_suffix())), |x| x.0 * x.1.factor())
}

#[allow(unused)]
mod test {
    // These lines are required despite warnings from rust-analyzer.
    use crate::value::{parse_suffix_number, NumberParser, SuffixNumberParser};
    use nom::Parser;

    #[test]
    fn test_suffix_number() {
        let value = parse_suffix_number().parse("32k").unwrap();
        assert!(value.1 == 32000.);

        // 科学计数法+后缀
        assert!(parse_suffix_number().parse("32e3m").unwrap().1 == 32.);

        // 易混淆：Meg (1e6) 和 m (1e-3)
        assert!(parse_suffix_number().parse("1meg").unwrap().1 == 1e6);
        assert!(parse_suffix_number().parse("1m").unwrap().1 == 1e-3);
    }

    #[test]
    fn test_number_double() {
        let parser = SuffixNumberParser {};
        assert!(parser.to_number("1").unwrap() == 1.);
        assert!(parser.to_number(".1").unwrap() == 0.1);
    }

    /// 数字解析器不区分大小写。
    #[test]
    fn test_number_case_insensitive() {
        let parser = SuffixNumberParser {};
        assert!(parser.to_number("1.5MEgohM").unwrap() == 1.5e6);
        assert!(parser.to_number("1.5M").unwrap() == 1.5e-3);
    }

    #[test]
    fn test_number_with_garbage() {
        let parser = SuffixNumberParser {};
        assert!(parser.to_number("1.5mohm").unwrap() == 1.5e-3);
        assert!(parser.to_number("1.5megohm").unwrap() == 1.5e6);
        assert!(parser.to_number("1.5ohm").unwrap() == 1.5);
    }
}

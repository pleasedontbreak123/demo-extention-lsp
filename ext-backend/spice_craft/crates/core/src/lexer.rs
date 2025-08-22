//! 可能装模作样写完了，希望不会有问题
//!
//! 如果有问题我也不知道

use crate::ast::Atom;

struct MergeLinesIter<T>
where
    T: Iterator<Item = Vec<Atom>>,
{
    iter: T,
    last: Option<Vec<Atom>>,
}

trait MergeLines<T>
where
    T: Iterator<Item = Vec<Atom>>,
{
    /// Merge lines which starts with `+`.
    fn merge_lines(self) -> MergeLinesIter<T>;
}

impl<T> MergeLines<T> for T
where
    T: Iterator<Item = Vec<Atom>>,
{
    fn merge_lines(mut self) -> MergeLinesIter<T> {
        let last = self.next();
        MergeLinesIter { iter: self, last }
    }
}

impl<T> Iterator for MergeLinesIter<T>
where
    T: Iterator<Item = Vec<Atom>>,
{
    type Item = Vec<Atom>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(mut last) = self.last.take() {
            while let Some(next) = self.iter.next() {
                if next.first().is_some_and(|x| x.raw == "+") {
                    last.extend(next.into_iter().skip(1));
                } else {
                    self.last = Some(next);
                    break;
                }
            }
            Some(last)
        } else {
            None
        }
    }
}

const SPECIAL_CHARS: &'static str = ")(][}{=:*<>!~&|+-^/";

/// Spice 词法解析器
///
/// ```
/// use spice_parser_core::lexer::SpiceLexer;
/// # use spice_parser_core::ast::Atom;
///
/// let tokens = SpiceLexer::tokenize("key=value");
/// assert_eq!(tokens, vec![vec![Atom::from("key"), Atom::from("="), Atom::from("value")]]);
/// ```
pub struct SpiceLexer;

impl SpiceLexer {
    /// 解析词法
    pub fn tokenize<'a>(code: &'a str) -> Vec<Vec<Atom>> {
        code.lines().enumerate()
            .map(|(loc,line)| {
                let i = line.as_ptr() as usize - code.as_ptr() as usize;

                // starting with `*` means the whole line is comment
                if line.trim().starts_with('*') {
                    return Vec::new();
                }

                let mut tokens: Vec<(usize, usize)> = Vec::new();
                let mut start = 0;
                // inline comment starts with `;`
                let line = line.split(';').next().unwrap();
                let mut skip = false;
                let mut is_quote = false;

                for (j, c) in line.char_indices() {
                    // eprintln!("{}: {}", j, c);
                    if skip {
                        skip = false;
                        continue;
                    }

                    if c == '"' {
                        if is_quote {
                            tokens.push((start - 1, j + 1));
                            is_quote = false;
                            start = j + 1;
                        } else {
                            is_quote = true;
                            start = j + 1;
                        }
                        continue;
                    }

                    // 如果正在 " 中
                    if is_quote {
                        continue;
                    }

                    let is_whitespace = c.is_whitespace() || c == ',';
                    let is_special_char = SPECIAL_CHARS.contains(c);

                    // 中断分词
                    // > | word+
                    // > | ~~~~^
                    if is_whitespace || is_special_char {
                        if start < j {
                            // check if current '-' or '+' is in number
                            let is_inside_number = {
                                let previous_chars = &line[start..j];
                                let is_start_with_dot_or_digit = previous_chars.starts_with(
                                    &['.', '0', '1', '2', '3', '4', '5', '6', '7', '8', '9'][..],
                                );
                                let is_after_e = previous_chars.ends_with(&['e', 'E'][..]);
                                let is_plus_or_minus = c == '+' || c == '-';
                                let is_followed_by_number = line[j..]
                                    .chars()
                                    .skip(1)
                                    .next()
                                    .is_some_and(|x| x.is_digit(10));

                                is_start_with_dot_or_digit
                                    && is_after_e
                                    && is_plus_or_minus
                                    && is_followed_by_number
                            };

                            // if current char is in number, we just skip it
                            if is_inside_number {
                                continue;
                            }

                            tokens.push((start, j));
                        }
                    }

                    // 优先匹配两个符号的情况
                    if is_special_char {
                        let next_2_chars = line[j..].chars().take(2).collect::<String>();
                        match &next_2_chars[..] {
                            "!=" | "==" | "<=" | ">=" | "**" => {
                                tokens.push((j, j + 2));
                                start = j + 2;
                                skip = true;
                            }
                            _ => {
                                tokens.push((j, j + 1));
                                start = j + 1;
                            }
                        }
                    }

                    // 空白则推迟符号开始时间
                    if is_whitespace {
                        start = j + 1;
                    }
                }
                if start < line.len() {
                    tokens.push((start, line.len()));
                }

                tokens
                    .into_iter()
                    .map(|(st, ed)| Atom::new(&line[st..ed], (st, ed), loc))
                    .collect()
            })
            .filter(|x| !x.is_empty())
            .merge_lines()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merge_lines_no_merge() {
        let lines = vec![
            vec![Atom::from("a")],
            vec![Atom::from("b")],
            vec![Atom::from("c")],
        ];
        let result: Vec<Vec<Atom>> = lines.into_iter().merge_lines().collect();

        assert_eq!(
            result,
            vec![
                vec![Atom::from("a")],
                vec![Atom::from("b")],
                vec![Atom::from("c")],
            ]
        );
    }

    #[test]
    fn test_merge_lines_with_merge() {
        let lines = vec![
            vec![Atom::from("a")],
            vec![Atom::from("+"), Atom::from("b")],
            vec![Atom::from("+"), Atom::from("c")],
            vec![Atom::from("d")],
        ];
        let result: Vec<Vec<Atom>> = lines.into_iter().merge_lines().collect();

        assert_eq!(
            result,
            vec![
                vec![Atom::from("a"), Atom::from("b"), Atom::from("c")],
                vec![Atom::from("d")],
            ]
        );
    }

    #[test]
    fn test_merge_lines_multiple_merge_blocks() {
        let lines = vec![
            vec![Atom::from("a")],
            vec![Atom::from("+"), Atom::from("b")],
            vec![Atom::from("c")],
            vec![Atom::from("+"), Atom::from("d")],
            vec![Atom::from("e")],
        ];
        let result: Vec<Vec<Atom>> = lines.into_iter().merge_lines().collect();

        assert_eq!(
            result,
            vec![
                vec![Atom::from("a"), Atom::from("b")],
                vec![Atom::from("c"), Atom::from("d")],
                vec![Atom::from("e")],
            ]
        );
    }

    #[test]
    fn test_merge_lines_single_line() {
        let lines = vec![vec![Atom::from("a")]];
        let result: Vec<Vec<Atom>> = lines.into_iter().merge_lines().collect();

        assert_eq!(result, vec![vec![Atom::from("a")]]);
    }

    #[test]
    fn test_merge_lines_empty_input() {
        let lines: Vec<Vec<Atom>> = vec![];
        let result: Vec<Vec<Atom>> = lines.into_iter().merge_lines().collect();

        assert_eq!(result, Vec::<Vec<Atom>>::new());
    }

    #[test]
    fn test_tokenize_basic_input() {
        let input = String::from("Hello (World), test {example} = 123");
        let expected = vec![vec![
            Atom::from("Hello"),
            Atom::from("("),
            Atom::from("World"),
            Atom::from(")"),
            Atom::from("test"),
            Atom::from("{"),
            Atom::from("example"),
            Atom::from("}"),
            Atom::from("="),
            Atom::from("123"),
        ]];

        let result = SpiceLexer::tokenize(&input);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_tokenize_only_special_characters() {
        let input = String::from("()=[] {}:");
        let expected = vec![vec![
            Atom::from("("),
            Atom::from(")"),
            Atom::from("="),
            Atom::from("["),
            Atom::from("]"),
            Atom::from("{"),
            Atom::from("}"),
            Atom::from(":"),
        ]];

        let result = SpiceLexer::tokenize(&input);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_tokenize_empty_string() {
        let input = String::from("");
        let result = SpiceLexer::tokenize(&input);
        assert_eq!(result, Vec::<Vec<Atom>>::new());
    }

    #[test]
    fn test_tokenize_utf_8() {
        let input = String::from(r#"你好 你所热爱+👍*就是你的生活//🇨🇿"#);
        let result = SpiceLexer::tokenize(&input);
        assert_eq!(
            result,
            vec![vec![
                Atom::from("你好"),
                Atom::from("你所热爱"),
                Atom::from("+"),
                Atom::from("👍"),
                Atom::from("*"),
                Atom::from("就是你的生活"),
                Atom::from("/"),
                Atom::from("/"),
                Atom::from("🇨🇿"),
            ],]
        );
    }

    #[test]
    fn test_tokenize_no_special_characters() {
        let input = String::from("This is a test string");
        let expected = vec![vec![
            Atom::from("This"),
            Atom::from("is"),
            Atom::from("a"),
            Atom::from("test"),
            Atom::from("string"),
        ]];

        let result = SpiceLexer::tokenize(&input);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_tokenize_hyphen_and_spaces() {
        let input = String::from("Hello -World, test {example} = 123");
        let expected = vec![vec![
            Atom::from("Hello"),
            Atom::from("-"),
            Atom::from("World"),
            Atom::from("test"),
            Atom::from("{"),
            Atom::from("example"),
            Atom::from("}"),
            Atom::from("="),
            Atom::from("123"),
        ]];

        let result = SpiceLexer::tokenize(&input);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_tokenize_long_operator() {
        let input = String::from("Hello!=World");
        let expected = vec![vec![
            Atom::from("Hello"),
            Atom::from("!="),
            Atom::from("World"),
        ]];

        let result = SpiceLexer::tokenize(&input);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_tokenize_long_operator_half() {
        let input = String::from("Hello!");
        let expected = vec![vec![Atom::from("Hello"), Atom::from("!")]];

        let result = SpiceLexer::tokenize(&input);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_tokenize_long_operator_chaos() {
        let input = String::from("a<<=>>==b");
        let expected = vec![vec![
            Atom::from("a"),
            Atom::from("<"),
            Atom::from("<="),
            Atom::from(">"),
            Atom::from(">="),
            Atom::from("="),
            Atom::from("b"),
        ]];

        let result = SpiceLexer::tokenize(&input);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_tokenize_smart_split_on_science_numbers() {
        let input = String::from("10e-2 e-4 .3e-4 1.3E-4 1.3e+4 1.3E4 ee+3");
        let expected = vec![vec![
            Atom::from("10e-2"),
            Atom::from("e"),
            Atom::from("-"),
            Atom::from("4"),
            Atom::from(".3e-4"),
            Atom::from("1.3E-4"),
            Atom::from("1.3e+4"),
            Atom::from("1.3E4"),
            Atom::from("ee"),
            Atom::from("+"),
            Atom::from("3"),
        ]];

        let result = SpiceLexer::tokenize(&input);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_tokenize_quotes() {
        let input = String::from(r#" "233" "114514 1919810" "" "\" "#);
        let expected = vec![vec![
            Atom::from("\"233\""),
            Atom::from("\"114514 1919810\""),
            Atom::from("\"\""),
            Atom::from("\"\\\""),
        ]];

        let result = SpiceLexer::tokenize(&input);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_tokenize_lines_models_collector() {
        let input = include_str!("../../../models/Collector/Collector.lib");
        let expected = vec![
            vec![
                Atom::from(".subckt"),
                Atom::from("Collector"),
                Atom::from("SA_HOp"),
                Atom::from("SA_HOn"),
                Atom::from("TrigenOut"),
                Atom::from("TR_HOp"),
                Atom::from("TR_HOn"),
                Atom::from("PWMp"),
                Atom::from("PWMn"),
                Atom::from("VCOp"),
                Atom::from("VCOn"),
                Atom::from("PEAK_p"),
                Atom::from("PEAK_n"),
                Atom::from("gnd"),
            ],
            vec![
                Atom::from("YSA_HO"),
                Atom::from("SA_HO"),
                Atom::from("TrigenOut"),
                Atom::from("gnd"),
                Atom::from("SA_HOp"),
                Atom::from("SA_HOn"),
                Atom::from("PARAM"),
                Atom::from(":"),
                Atom::from("FS"),
                Atom::from("="),
                Atom::from("5meg"),
                Atom::from("TACQ"),
                Atom::from("="),
                Atom::from("1E-9"),
                Atom::from("DV"),
                Atom::from("="),
                Atom::from("0.05"),
            ],
            vec![
                Atom::from("VCTR"),
                Atom::from("CTR"),
                Atom::from("gnd"),
                Atom::from("DC"),
                Atom::from("3.3"),
            ],
            vec![
                Atom::from("YTR_HO"),
                Atom::from("TR_HO"),
                Atom::from("TrigenOut"),
                Atom::from("gnd"),
                Atom::from("TR_HOp"),
                Atom::from("TR_HOn"),
                Atom::from("CTR"),
                Atom::from("PARAM"),
                Atom::from(":"),
                Atom::from("VTH"),
                Atom::from("="),
                Atom::from("1"),
            ],
            vec![
                Atom::from("YTRIGEN"),
                Atom::from("TRIGEN"),
                Atom::from("TrigenOut"),
                Atom::from("gnd"),
                Atom::from("PARAM"),
                Atom::from(":"),
                Atom::from("rdu"),
                Atom::from("="),
                Atom::from("1.0e-3"),
                Atom::from("tdel"),
                Atom::from("="),
                Atom::from("0.0001"),
                Atom::from("v0"),
                Atom::from("="),
                Atom::from("1.0"),
            ],
            vec![
                Atom::from("YPWM"),
                Atom::from("PWM"),
                Atom::from("TrigenOut"),
                Atom::from("gnd"),
                Atom::from("PWMp"),
                Atom::from("PWMn"),
                Atom::from("PARAM"),
                Atom::from(":"),
                Atom::from("PVMAX"),
                Atom::from("="),
                Atom::from("5.0"),
                Atom::from("PVMIN"),
                Atom::from("="),
                Atom::from("1.0"),
            ],
            vec![
                Atom::from("YVCO"),
                Atom::from("VCO"),
                Atom::from("TrigenOut"),
                Atom::from("gnd"),
                Atom::from("VCOp"),
                Atom::from("VCOn"),
                Atom::from("PARAM"),
                Atom::from(":"),
                Atom::from("V1"),
                Atom::from("="),
                Atom::from("3.3"),
                Atom::from("VOFF"),
                Atom::from("="),
                Atom::from("0"),
            ],
            vec![
                Atom::from("YPEAK_D"),
                Atom::from("PEAK_D"),
                Atom::from("TrigenOut"),
                Atom::from("gnd"),
                Atom::from("PEAK_p"),
                Atom::from("PEAK_n"),
                Atom::from("CTR"),
                Atom::from("PARAM"),
                Atom::from(":"),
                Atom::from("VTH"),
                Atom::from("="),
                Atom::from("1"),
                Atom::from("SLR"),
                Atom::from("="),
                Atom::from("10"),
                Atom::from("RSLR"),
                Atom::from("="),
                Atom::from("1"),
            ],
            vec![
                Atom::from("YLEV_D"),
                Atom::from("LEV_D"),
                Atom::from("PWMout"),
                Atom::from("gnd"),
                Atom::from("LEV_p"),
                Atom::from("LEV_n"),
                Atom::from("PARAM"),
                Atom::from(":"),
                Atom::from("V0"),
                Atom::from("="),
                Atom::from("1"),
                Atom::from("V1"),
                Atom::from("="),
                Atom::from("5"),
                Atom::from("VRL"),
                Atom::from("="),
                Atom::from("2.4"),
                Atom::from("VRU"),
                Atom::from("="),
                Atom::from("2.6"),
                Atom::from("TR"),
                Atom::from("="),
                Atom::from("1us"),
                Atom::from("TF"),
                Atom::from("="),
                Atom::from("1us"),
            ],
            vec![
                Atom::from("Vpwm"),
                Atom::from("PWMout"),
                Atom::from("gnd"),
                Atom::from("AC"),
                Atom::from("0.0"),
                Atom::from("PULSE"),
                Atom::from("("),
                Atom::from("0.0"),
                Atom::from("5.0"),
                Atom::from("0.0"),
                Atom::from("0.01E-3"),
                Atom::from("0.01E-3"),
                Atom::from("0.1E-3"),
                Atom::from("0.2E-3"),
                Atom::from(")"),
            ],
            vec![Atom::from(".ends")],
        ];

        let result = SpiceLexer::tokenize(input);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_tokenize_lines_models_ctr_sourse() {
        let input = include_str!("../../../models/ctr_sourse/ctr_sourse.cir");
        let expected = vec![
            vec![Atom::from(".option"), Atom::from("uselastdef")],
            vec![
                Atom::from(".MODEL"),
                Atom::from("1N4001"),
                Atom::from("D"),
                Atom::from("("),
                Atom::from("IS"),
                Atom::from("="),
                Atom::from("11.4956P"),
                Atom::from("RS"),
                Atom::from("="),
                Atom::from("0.114"),
                Atom::from("N"),
                Atom::from("="),
                Atom::from("1.321"),
                Atom::from("CJO"),
                Atom::from("="),
                Atom::from("36.1697P"),
                Atom::from("VJ"),
                Atom::from("="),
                Atom::from("0.583"),
                Atom::from("M"),
                Atom::from("="),
                Atom::from("0.464"),
                Atom::from("BV"),
                Atom::from("="),
                Atom::from("50"),
                Atom::from("IBV"),
                Atom::from("="),
                Atom::from("50N"),
                Atom::from(")"),
            ],
            vec![
                Atom::from(".model"),
                Atom::from("a2d_eldo"),
                Atom::from("a2d"),
                Atom::from("mode"),
                Atom::from("="),
                Atom::from("std_logic"),
            ],
            vec![
                Atom::from(".model"),
                Atom::from("d2a_eldo"),
                Atom::from("d2a"),
                Atom::from("mode"),
                Atom::from("="),
                Atom::from("std_logic"),
                Atom::from("TRISE"),
                Atom::from("="),
                Atom::from("50e-12"),
                Atom::from("TFALL"),
                Atom::from("="),
                Atom::from("50e-12"),
            ],
            vec![Atom::from(".defhook"), Atom::from("a2d_eldo")],
            vec![Atom::from(".defhook"), Atom::from("d2a_eldo")],
            vec![
                Atom::from("V1I251"),
                Atom::from("VIN"),
                Atom::from("0"),
                Atom::from("DC"),
                Atom::from("5.0"),
                Atom::from("AC"),
                Atom::from("0.0"),
                Atom::from("0.0"),
            ],
            vec![
                Atom::from("V1I175"),
                Atom::from("VCTR"),
                Atom::from("0"),
                Atom::from("AC"),
                Atom::from("0.0"),
                Atom::from("SIN"),
                Atom::from("("),
                Atom::from("1.0"),
                Atom::from("1.0"),
                Atom::from("1.0E3"),
                Atom::from("0.0"),
                Atom::from("0.0"),
                Atom::from("0.0"),
                Atom::from(")"),
            ],
            vec![
                Atom::from("E1I242"),
                Atom::from("VIN"),
                Atom::from("VOUT"),
                Atom::from("VCTR"),
                Atom::from("0"),
                Atom::from("1.0"),
            ],
            vec![
                Atom::from("G1I266"),
                Atom::from("IIN"),
                Atom::from("IOUT"),
                Atom::from("VCTR"),
                Atom::from("0"),
                Atom::from("1.0"),
            ],
            vec![
                Atom::from("R1I176"),
                Atom::from("VOUT"),
                Atom::from("0"),
                Atom::from("1.0E3"),
            ],
            vec![
                Atom::from("R1I313"),
                Atom::from("VIN"),
                Atom::from("IIN"),
                Atom::from("1.0E3"),
            ],
            vec![
                Atom::from("Dd1"),
                Atom::from("0"),
                Atom::from("IOUT"),
                Atom::from("1N4001"),
            ],
            vec![
                Atom::from("H1I321"),
                Atom::from("VIN"),
                Atom::from("VOUT2"),
                Atom::from("Vsin"),
                Atom::from("1.0"),
            ],
            vec![
                Atom::from("Vsin"),
                Atom::from("ICTR"),
                Atom::from("0"),
                Atom::from("AC"),
                Atom::from("0.0"),
                Atom::from("SIN"),
                Atom::from("("),
                Atom::from("0.0"),
                Atom::from("1.0"),
                Atom::from("1.0E3"),
                Atom::from("0.0"),
                Atom::from("0.0"),
                Atom::from("0.0"),
                Atom::from(")"),
            ],
            vec![
                Atom::from("R1I347"),
                Atom::from("VOUT2"),
                Atom::from("0"),
                Atom::from("1.0E3"),
            ],
            vec![
                Atom::from("Rr1"),
                Atom::from("0"),
                Atom::from("ICTR"),
                Atom::from("1.0E3"),
            ],
            vec![
                Atom::from("F1I452"),
                Atom::from("IIN2"),
                Atom::from("IOUT2"),
                Atom::from("Vsin"),
                Atom::from("1.0"),
            ],
            vec![
                Atom::from("Rout"),
                Atom::from("IOUT2"),
                Atom::from("0"),
                Atom::from("1.0E3"),
            ],
            vec![
                Atom::from("I1I506"),
                Atom::from("IIN2"),
                Atom::from("N1N492"),
                Atom::from("DC"),
                Atom::from("5.0"),
                Atom::from("AC"),
                Atom::from("1.0"),
            ],
            vec![
                Atom::from("R1I493"),
                Atom::from("IIN2"),
                Atom::from("N1N492"),
                Atom::from("1.0E3"),
            ],
        ];

        let result = SpiceLexer::tokenize(input);
        assert_eq!(result, expected);
    }
}

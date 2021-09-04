use std::{borrow::Cow, mem};

enum EscapingState {
    /// "\"
    Start,
    /// "\x"
    AsciiCharCodeStart,
    /// ""
    AsciiCharCode(u8),
    /// \u
    UnicodeStart,
    /// \u{
    UnicodeStartBrace,
    /// \u{100
    Unicode(u32),
}

enum QuotedState {
    Normal,
    Escaping(EscapingState),
}

enum ValueState {
    /// (last index)
    Raw,
    // DoubleEscaped(EscapedState<'a>),
    // SingleEscaped(EscapedState<'a>),
    Quoted {
        quote: char,
        current: String,
        state: QuotedState,
    },
    QuoteEnd(String),
}

enum State<'a> {
    /// (whitespace length)
    None(usize),
    /// (start index, exclusive end index)
    Key(usize, usize),
    /// (key, index of '=')
    KeyAndEqual(&'a str, usize),
    KeyAndValue {
        key: &'a str,
        /// (start index, state)
        value_start: usize,
        value_end: usize,
        value_state: ValueState,
    },
    KeyAndValueAndProgram {
        key: &'a str,
        value: Cow<'a, str>,
        program_start: usize,
    },
}

fn try_hex_to_u8(hex: char) -> Option<u8> {
    Some(match hex {
        '0' => 0,
        '1' => 1,
        '2' => 2,
        '3' => 3,
        '4' => 4,
        '5' => 5,
        '6' => 6,
        '7' => 7,
        '8' => 8,
        '9' => 9,
        'A' | 'a' => 10,
        'B' | 'b' => 11,
        'C' | 'c' => 12,
        'D' | 'd' => 13,
        'E' | 'e' => 14,
        'F' | 'f' => 15,
        _ => return None,
    })
}

fn match_one_env(program: &str) -> (Option<(&str, Cow<str>)>, &str) {
    let mut state_data = State::None(0);
    let state = &mut state_data;

    for c in program.chars() {
        match state {
            State::None(len) => {
                if c.is_whitespace() {
                    *len += c.len_utf8();
                } else {
                    *state = State::Key(*len, *len + c.len_utf8());
                }
            }
            State::Key(start, end) => {
                if c == '=' {
                    let end = *end;
                    let key = &program[*start..end];
                    *state = State::KeyAndEqual(key, end);
                } else if c.is_whitespace() {
                    break;
                } else {
                    *end += c.len_utf8();
                }
            }
            State::KeyAndEqual(key, equal_index) => {
                let start = *equal_index + 1;

                let value_state = match c {
                    '"' | '\'' => ValueState::Quoted {
                        quote: c,
                        current: String::with_capacity(program.len() - start),
                        state: QuotedState::Normal,
                    },
                    c if c.is_whitespace() => break,
                    _ => ValueState::Raw,
                };

                *state = State::KeyAndValue {
                    key,
                    value_start: start,
                    value_end: start + c.len_utf8(),
                    value_state,
                };
            }
            State::KeyAndValue {
                key,
                value_start,
                value_state,
                value_end,
            } => {
                match value_state {
                    ValueState::Raw => {
                        match c {
                            '=' | '"' | '\'' => break,
                            _ => {
                                if c.is_whitespace() {
                                    let end = *value_end;
                                    *state = State::KeyAndValueAndProgram {
                                        key,
                                        value: Cow::Borrowed(&program[*value_start..end]),
                                        program_start: end + c.len_utf8(),
                                    };
                                } else {
                                    *value_end += c.len_utf8();
                                }
                            }
                        };
                    }
                    ValueState::Quoted {
                        quote,
                        current,
                        state: quoted_state,
                    } => {
                        *value_end += c.len_utf8();

                        let quote = *quote;
                        match quoted_state {
                            QuotedState::Normal => {
                                if c == quote {
                                    let current = mem::take(current);
                                    *value_state = ValueState::QuoteEnd(current);
                                } else if c == '\\' {
                                    *quoted_state = QuotedState::Escaping(EscapingState::Start);
                                } else {
                                    current.push(c);
                                }
                            }
                            QuotedState::Escaping(escaping_state) => {
                                match escaping_state {
                                    EscapingState::Start => {
                                        enum EscapedChar {
                                            OneChar(char),
                                            AsciiCode,
                                            Unicode,
                                            Invalid,
                                        }
                                        use EscapedChar::*;

                                        let escaped_char = match c {
                                            'n' => OneChar('\n'),
                                            'r' => OneChar('\r'),
                                            't' => OneChar('\t'),
                                            '\\' => OneChar('\\'),
                                            '0' => OneChar('\0'),
                                            'x' => AsciiCode,
                                            'u' => Unicode,
                                            c if c == quote => OneChar(quote),
                                            _ => Invalid,
                                        };

                                        match escaped_char {
                                            OneChar(escaped_char) => {
                                                current.push(escaped_char);
                                                *quoted_state = QuotedState::Normal;
                                            }
                                            AsciiCode => {
                                                *escaping_state = EscapingState::AsciiCharCodeStart;
                                            }
                                            Unicode => {
                                                *escaping_state = EscapingState::UnicodeStart;
                                            }
                                            Invalid => break,
                                        }
                                    }
                                    EscapingState::AsciiCharCodeStart => {
                                        let n = try_hex_to_u8(c);
                                        if let Some(n) = n {
                                            if n <= 7 {
                                                *escaping_state = EscapingState::AsciiCharCode(n);
                                            } else {
                                                break;
                                            }
                                        } else {
                                            break;
                                        }
                                    }
                                    EscapingState::AsciiCharCode(n) => {
                                        let n2 = try_hex_to_u8(c);
                                        if let Some(n2) = n2 {
                                            let code = *n * 16 + n2;
                                            if code > 127 {
                                                break;
                                            }
                                            if let Some(c) = char::from_u32(code.into()) {
                                                current.push(c);
                                                *quoted_state = QuotedState::Normal;
                                            } else {
                                                break;
                                            }
                                        } else {
                                            break;
                                        }
                                    }
                                    EscapingState::UnicodeStart => {
                                        if c == '{' {
                                            *escaping_state = EscapingState::UnicodeStartBrace;
                                        } else {
                                            break;
                                        }
                                    }
                                    EscapingState::UnicodeStartBrace => {
                                        let n = try_hex_to_u8(c);
                                        if let Some(n) = n {
                                            *escaping_state = EscapingState::Unicode(n.into());
                                        } else {
                                            break;
                                        }
                                    }
                                    EscapingState::Unicode(n) => {
                                        if c == '}' {
                                            if let Some(unescaped) = char::from_u32(*n) {
                                                current.push(unescaped);
                                                *quoted_state = QuotedState::Normal;
                                            }
                                        } else {
                                            let n2 = try_hex_to_u8(c);

                                            if let Some(n2) = n2 {
                                                let code = (*n)
                                                    .checked_mul(16)
                                                    .and_then(|v| v.checked_add(n2.into()));
                                                if let Some(code) = code {
                                                    *n = code
                                                } else {
                                                    break;
                                                }
                                            } else {
                                                break;
                                            }
                                        }
                                    }
                                };
                            }
                        };
                    }
                    ValueState::QuoteEnd(unescaped_str) => {
                        let unescaped_str = mem::take(unescaped_str);
                        if c.is_whitespace() {
                            *state = State::KeyAndValueAndProgram {
                                key,
                                value: Cow::Owned(unescaped_str),
                                program_start: *value_end + c.len_utf8(),
                            };
                        } else {
                            // invalid: KEY="VALUE"echo 1
                            break;
                        }
                    }
                };
            }
            State::KeyAndValueAndProgram { .. } => {
                break;
            }
        };
    }

    match state_data {
        State::KeyAndValueAndProgram {
            key,
            value,
            program_start,
        } => (Some((key, value)), &program[program_start..].trim()),
        _ => (None, program.trim()),
    }
}

/// match program and envs from simple shell script.
///
/// For example:
/// ```
/// # use runcc::match_program_with_envs;
/// assert_eq!(match_program_with_envs("cargo run"), ("cargo run", None));
/// assert_eq!(match_program_with_envs("MY_KEY=MY_VALUE cargo run"), ("cargo run", Some(vec![("MY_KEY", "MY_VALUE".into())])));
/// assert_eq!(match_program_with_envs("MY_KEY=MY_VALUE ANOTHER_KEY='ANOTHER VALUE' cargo run"), ("cargo run", Some(vec![("MY_KEY", "MY_VALUE".into()), ("ANOTHER_KEY", "ANOTHER VALUE".into())])));
/// ```
///
/// You can escape special characters in quoted string.
/// The escape syntax is the same as [Rust string literal escape syntax](https://doc.rust-lang.org/reference/tokens.html#characters-and-strings)
///
/// ```
/// # use runcc::match_program_with_envs;
/// assert_eq!(match_program_with_envs(r#"MY_KEY="\n \x25 \u{26}" cargo run"#), ("cargo run", Some(vec![("MY_KEY", "\n % &".into())])));
/// ```
pub fn match_program_with_envs(mut program: &str) -> (&str, Option<Vec<(&str, Cow<str>)>>) {
    let mut envs = vec![];

    loop {
        let (env, new_program) = match_one_env(program);

        if let Some(env) = env {
            program = new_program;
            envs.push(env);
        } else {
            return (new_program, if envs.len() > 0 { Some(envs) } else { None });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::match_one_env;

    #[test]
    fn match_one_env_no_env() {
        assert_eq!(match_one_env(""), (None, ""));
        assert_eq!(match_one_env("   "), (None, ""));
        assert_eq!(match_one_env("cargo"), (None, "cargo"));
        assert_eq!(match_one_env("cargo run"), (None, "cargo run"));
        assert_eq!(match_one_env("cargo   run"), (None, "cargo   run"));
        assert_eq!(match_one_env("   cargo run"), (None, "cargo run"));
        assert_eq!(match_one_env("cargo run   "), (None, "cargo run"));
        assert_eq!(match_one_env(" cargo   run  "), (None, "cargo   run"));
    }

    #[test]
    fn match_one_env_fail() {
        for program in [
            "A=",
            "A= cargo run",
            r#"A="\ ""#,
            r"A='B\xa' cargo run",
            r"A='B\x80' cargo run",
            r"A='B\u' cargo run",
            r"A='B\u1' cargo run",
            r"A='B\u{110000}' cargo run",
            r"A='B\u{fffffffff}' cargo run",
            r"A='B cargo run",
            r"A='B'cargo run",
        ] {
            assert_eq!(match_one_env(program), (None, program));
        }
    }

    #[test]
    fn match_one_env_ok() {
        for (program, k, v, result_program) in [
            ("K=V cargo", "K", "V", "cargo"),
            ("  K=V  cargo   ", "K", "V", "cargo"),
            ("  K=V  cargo  run ", "K", "V", "cargo  run"),
            ("k=v k2=v2 cargo  run ", "k", "v", "k2=v2 cargo  run"),
            ("k='v k2=v2' cargo  run ", "k", "v k2=v2", "cargo  run"),
            (
                r#"k='v\n\t\r\0\'" k2=v2' cargo  run "#,
                "k",
                "v\n\t\r\0'\" k2=v2",
                "cargo  run",
            ),
            (
                r#"k="v\n\t\r\0'\" k2=v2" cargo  run "#,
                "k",
                "v\n\t\r\0'\" k2=v2",
                "cargo  run",
            ),
            (
                r#"SOME_KEY="\x26\x20\x7f" cargo run"#,
                "SOME_KEY",
                "& \x7f",
                "cargo run",
            ),
            (
                r#"SOME_KEY="\u{20}\u{1F600}\u{10ffff}" cargo run"#,
                "SOME_KEY",
                " \u{1F600}\u{10ffff}",
                "cargo run",
            ),
        ] {
            assert_eq!(
                match_one_env(program),
                (Some((k, v.into())), result_program)
            );
        }
    }
}

use std::borrow::Cow;

use lazy_static::lazy_static;
use regex::{Captures, Regex, Replacer};

fn replace_all_cow<'a, R: Replacer>(
    cow: Cow<'a, str>,
    regex: &Regex,
    replacement: R,
) -> Cow<'a, str> {
    match cow {
        Cow::Borrowed(s) => regex.replace_all(s, replacement),
        Cow::Owned(s) => Cow::Owned(regex.replace_all(&s, replacement).into_owned()),
    }
}

/// process ansi escapes in a line to not clear the output before
/// see: https://en.wikipedia.org/wiki/ANSI_escape_code#CSI_(Control_Sequence_Introducer)_sequences
pub(crate) fn process_ansi_escape_line<'a>(start_len: usize, line: &'a str) -> Cow<'a, str> {
    if !line.contains("\x1B[") {
        return line.into();
    }

    // remove "clear from cursor to beginning of the line"
    // TODO: simulate clear logic
    lazy_static! {
        static ref RE_CLEAR_LINE: Regex = Regex::new(r"\x1B\x5B[12]K").unwrap();
    }
    let line = RE_CLEAR_LINE.replace_all(line, "");

    // remove: Cursor Up / Down; Erase in Display; Cursor Next / Previous Line; Cursor Position
    lazy_static! {
        static ref RE_TO_REMOVE: Regex = Regex::new(r"\x1B\x5B\d*(?:[ABJEF]|(?:;\d*H))").unwrap();
    }
    let line = replace_all_cow(line, &RE_TO_REMOVE, "");

    // Cursor Horizontal Absolute
    lazy_static! {
        static ref RE_CURSOR_HORIZONTAL_ABS: Regex = Regex::new(r"\x1B\x5B(\d*)G").unwrap();
    }

    let line = replace_all_cow(line, &RE_CURSOR_HORIZONTAL_ABS, |caps: &Captures| {
        let num = &caps[1];
        let num: usize = if num.is_empty() {
            1
        } else {
            usize::from_str_radix(num, 10).unwrap()
        };

        let num = num + start_len;

        format!("\x1B\x5B{}G", num)
    });

    // Cursor Position
    lazy_static! {
        static ref RE_CURSOR_POSITION: Regex = Regex::new(r"\x1B\x5B(\d*);(\d*)H").unwrap();
    }
    let line = replace_all_cow(line, &RE_CURSOR_POSITION, |caps: &Captures| {
        let num = &caps[2];
        let num: usize = if num.is_empty() {
            1
        } else {
            usize::from_str_radix(num, 10).unwrap()
        };

        let num = num + start_len;

        format!("\x1B\x5B{}G", num)
    });

    line
}

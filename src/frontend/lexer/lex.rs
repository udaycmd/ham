use crate::frontend::lexer::{
    pos::{File, Position, SourcePosition},
    token::{Tok, get_ident_or_keyword},
};
use std::str::from_utf8;

const BOM: char = '\u{FEFF}';

#[inline(always)]
fn hex_digit_value(ch: char) -> u32 {
    ch.to_digit(16).unwrap_or(16)
}

pub struct Lexer<'a, E>
where
    E: Fn(String, SourcePosition),
{
    pub file: &'a mut File,
    src: &'a [u8],
    parse_comment: bool,
    err_cb: E,
    err_count: u64,
    cc: Option<char>,
    offset: usize,
    read_offset: usize,
}

impl<'a, E> Lexer<'a, E>
where
    E: Fn(String, SourcePosition),
{
    pub fn new(file: &'a mut File, src: &'a [u8], parse_comment: bool, err_cb: E) -> Self {
        let mut lexer = Self {
            file,
            src,
            parse_comment,
            err_cb,
            err_count: 0,
            cc: Some(' '),
            offset: 0,
            read_offset: 0,
        };

        lexer.next();
        if lexer.cc == Some(BOM) {
            lexer.next(); // strip BOM
        }

        lexer
    }

    pub fn scan(&mut self) -> (Tok, String, Position) {
        let mut lit = "";
        let mut tok = Tok::Invalid;

        self.scan_whitespace();

        let pos = self.file.to_set_pos(self.offset);

        let c = match self.cc {
            Some(ch) => ch,
            None => {
                return (Tok::Eof, Tok::Eof.to_str().to_owned(), pos);
            }
        };

        if c == '_' || c.is_alphabetic() {
            lit = self.scan_ident();
            tok = get_ident_or_keyword(lit);
        } else if c.is_ascii_digit() || (c == '.' && self.peek().is_ascii_digit()) {
            (tok, lit) = self.scan_number();
        } else {
            self.next();

            match c {
                '"' => {
                    tok = Tok::StringLiteral;
                    lit = self.scan_string();
                }

                '\'' => {
                    tok = Tok::CharLiteral;
                    lit = self.scan_char();
                }

                '#' => {
                    let comment = self.scan_comment();
                    if !self.parse_comment {
                        return self.scan();
                    }

                    tok = Tok::Comment;
                    lit = comment;
                }

                '?' => tok = Tok::Question,
                '(' => tok = Tok::LeftParen,
                ')' => tok = Tok::RightParen,
                '[' => tok = Tok::LeftBracket,
                ']' => tok = Tok::RightBracket,
                '{' => tok = Tok::LeftBrace,
                '}' => tok = Tok::RightBrace,
                ',' => tok = Tok::Comma,
                ';' => tok = Tok::Semicolon,
                ':' => tok = Tok::Colon,

                '=' => {
                    tok = Tok::Assign;
                    if self.cc == Some('=') {
                        tok = Tok::Equals;
                        self.next();
                    }
                }

                '+' => {
                    tok = Tok::Plus;
                    if self.cc == Some('=') {
                        tok = Tok::PlusEq;
                        self.next();
                    }
                }

                '*' => {
                    tok = Tok::Star;
                    if self.cc == Some('=') {
                        tok = Tok::StarEq;
                        self.next();
                    }
                }

                '/' => {
                    tok = Tok::Slash;
                    if self.cc == Some('=') {
                        tok = Tok::SlashEq;
                        self.next();
                    }
                }

                '%' => {
                    tok = Tok::Percent;
                    if self.cc == Some('=') {
                        tok = Tok::PercentEq;
                        self.next();
                    }
                }

                '!' => {
                    tok = Tok::Bang;
                    if self.cc == Some('=') {
                        tok = Tok::BangEq;
                        self.next();
                    }
                }

                '^' => {
                    tok = Tok::Xor;
                    if self.cc == Some('=') {
                        tok = Tok::XorEq;
                        self.next();
                    }
                }

                '-' => {
                    tok = Tok::Minus;
                    if self.cc == Some('=') {
                        tok = Tok::MinusEq;
                        self.next();
                    }
                }

                '>' => {
                    tok = Tok::Greater;
                    match self.cc {
                        Some('>') => {
                            tok = Tok::RightShift;
                            self.next();
                            if self.cc == Some('=') {
                                tok = Tok::RshiftEq;
                                self.next();
                            }
                        }
                        Some('=') => {
                            tok = Tok::GreaterEq;
                            self.next();
                        }
                        _ => {}
                    }
                }

                '<' => {
                    tok = Tok::Less;
                    match self.cc {
                        Some('<') => {
                            tok = Tok::LeftShift;
                            self.next();
                            if self.cc == Some('=') {
                                tok = Tok::LshiftEq;
                                self.next();
                            }
                        }
                        Some('=') => {
                            tok = Tok::LessEq;
                            self.next();
                        }
                        _ => {}
                    }
                }

                '&' => {
                    tok = Tok::Ampersand;
                    match self.cc {
                        Some('=') => {
                            tok = Tok::AndEq;
                            self.next();
                        }
                        Some('&') => {
                            tok = Tok::And;
                            self.next();
                        }
                        _ => {}
                    }
                }

                '|' => {
                    tok = Tok::Pipe;
                    match self.cc {
                        Some('=') => {
                            tok = Tok::OrEq;
                            self.next();
                        }
                        Some('|') => {
                            tok = Tok::Or;
                            self.next();
                        }
                        _ => {}
                    }
                }

                '.' => {
                    tok = Tok::Dot;
                    if self.cc == Some('.') {
                        self.next();
                        if self.cc == Some('.') {
                            tok = Tok::DotDotDot;
                            self.next();
                        } else {
                            self.err("invalid ellipsis mark", self.file.to_file_offset(pos));
                        }
                    }
                }

                _ => {
                    if c != BOM {
                        self.err(
                            "unexpected unicode character",
                            self.file.to_file_offset(pos),
                        );
                    };

                    return (tok, c.to_string(), pos);
                }
            }
        }

        if tok.is_operator() || tok.is_punctuator() {
            let lit = tok.to_str().to_owned();
            (tok, lit, pos)
        } else {
            (tok, lit.to_owned(), pos)
        }
    }

    #[inline]
    fn err(&mut self, msg: &str, offset: usize) {
        (self.err_cb)(
            msg.to_owned(),
            self.file.source_pos(self.file.to_set_pos(offset)),
        );
        self.err_count += 1;
    }

    fn next(&mut self) {
        if self.read_offset < self.src.len() {
            self.offset = self.read_offset;

            if self.cc == Some('\n') {
                self.file.add_line(self.offset);
            }

            let tail = &self.src[self.read_offset..];
            let (rune, width) = if tail[0] < 128 {
                (tail[0] as char, 1)
            } else {
                let max = tail.len().min(4);
                match from_utf8(&tail[..max]) {
                    Ok(s) => {
                        let c = s.chars().next().unwrap();
                        (c, c.len_utf8())
                    }
                    Err(e) => {
                        let valid_up_to = e.valid_up_to();
                        if valid_up_to > 0 {
                            let s = str::from_utf8(&tail[..valid_up_to]).unwrap();
                            let c = s.chars().next().unwrap();
                            (c, c.len_utf8())
                        } else {
                            (char::REPLACEMENT_CHARACTER, 1)
                        }
                    }
                }
            };

            if rune == '\0' {
                self.err("unexpected null character", self.offset);
            } else if rune == char::REPLACEMENT_CHARACTER && width == 1 {
                self.err("illegal unicode character in lexer stream", self.offset);
            } else if rune == BOM && self.offset > 0 {
                self.err("unexpected UTF_8 BOM", self.offset);
            }

            self.read_offset += width;
            self.cc = Some(rune);
        } else {
            self.offset = self.src.len();
            if self.cc == Some('\n') {
                self.file.add_line(self.offset);
            }

            self.cc = None; // eof
        }
    }

    #[inline]
    fn peek(&self) -> u8 {
        self.src.get(self.read_offset).copied().unwrap_or(0)
    }

    #[inline]
    fn scan_ident(&mut self) -> &'a str {
        let prev_offset = self.offset;
        while let Some(c) = self.cc {
            if c.is_alphabetic() || c.is_numeric() {
                self.next();
            } else {
                break;
            }
        }

        from_utf8(&self.src[prev_offset..self.offset]).unwrap()
    }

    #[inline]
    fn scan_whitespace(&mut self) {
        while let Some(c) = self.cc {
            if c.is_whitespace() {
                self.next();
            } else {
                break;
            }
        }
    }

    #[inline]
    fn scan_comment(&mut self) -> &'a str {
        let prev_offset = self.offset - 1;
        while let Some(c) = self.cc {
            if c == '\n' {
                break;
            }
            self.next();
        }

        from_utf8(&self.src[prev_offset..self.offset]).unwrap()
    }

    #[inline]
    fn scan_digit_seq(&mut self, base: u32) {
        while let Some(c) = self.cc {
            if c == '_' || hex_digit_value(c) < base {
                self.next()
            } else {
                break;
            }
        }
    }

    fn scan_number(&mut self) -> (Tok, &'a str) {
        let mut tok = Tok::IntegerLiteral;
        let prev_offset = self.offset;
        let mut base = 10;

        let peek_char = (self.peek() as char).to_ascii_lowercase();
        if self.cc == Some('0') && peek_char == 'b' {
            base = 2;
            self.next();
            self.next();
        } else if self.cc == Some('0') && peek_char == 'o' {
            base = 8;
            self.next();
            self.next();
        } else if self.cc == Some('0') && peek_char == 'x' {
            base = 16;
            self.next();
            self.next();
        }

        if base != 10 && hex_digit_value(self.peek() as char) == 16 {
            self.err("no digits after base specifier", prev_offset);
            return (tok, from_utf8(&self.src[prev_offset..self.offset]).unwrap());
        }

        // scan whole number
        self.scan_digit_seq(base);

        // scan fractional
        if self.cc == Some('.') && base == 10 {
            tok = Tok::RealLiteral;
            self.next();
            self.scan_digit_seq(base)
        }

        // scan exponent
        if self.cc == Some('e') || self.cc == Some('E') {
            tok = Tok::RealLiteral;
            self.next();

            // scan exponent sign
            if self.cc == Some('-') || self.cc == Some('+') {
                self.next()
            }

            let exponent_offset = self.offset;
            self.scan_digit_seq(10);

            if exponent_offset == self.offset {
                self.err("no digits after exponent", exponent_offset);
            }
        }

        (tok, from_utf8(&self.src[prev_offset..self.offset]).unwrap())
    }

    fn scan_escape(&mut self, quote: char) -> bool {
        let prev_offset = self.offset;
        let mut n;
        let base;
        let max;

        let c = match self.cc {
            Some(c) => c,
            None => {
                self.err("unterminated escape sequence", prev_offset);
                return false;
            }
        };

        match c {
            'a' | 'b' | 'f' | 'n' | 'r' | 't' | 'v' | '\\' => {
                self.next();
                return true;
            }

            c if c == quote => {
                self.next();
                return true;
            }

            '0'..='7' => {
                n = 3;
                base = 8;
                max = 255;
            }

            'x' => {
                self.next();
                n = 2;
                base = 16;
                max = 255;
            }

            'u' => {
                self.next();
                n = 4;
                base = 16;
                max = char::MAX as u32;
            }

            'U' => {
                self.next();
                n = 8;
                base = 16;
                max = char::MAX as u32;
            }

            _ => {
                self.err("unknown escape sequence", prev_offset);
                return false;
            }
        }

        let mut x = 0;

        while n > 0 {
            let d = if let Some(ch) = self.cc {
                hex_digit_value(ch)
            } else {
                self.err("unterminated escape sequence", self.offset);
                return false;
            };

            if d >= base {
                let msg = if self.cc.is_none() {
                    "unterminated escape sequence"
                } else {
                    "illegal unicode escape sequence"
                };

                self.err(msg, self.offset);
                return false;
            }

            x = x * base + d;
            self.next();
            n -= 1;
        }

        if x > max || (0xD800..0xE000).contains(&x) {
            self.err("illegal unicode escape sequence", prev_offset);
            return false;
        }

        true
    }

    fn scan_string(&mut self) -> &'a str {
        let prev_offset = self.offset - 1;
        loop {
            let ch = self.cc;

            if ch == Some('\n') || ch.is_none() {
                self.err("unterminated string literal", prev_offset);
                break;
            }

            self.next();

            if ch == Some('"') {
                break;
            }

            if ch == Some('\\') {
                self.scan_escape('"');
            }
        }

        from_utf8(&self.src[prev_offset..self.offset]).unwrap()
    }

    fn scan_char(&mut self) -> &'a str {
        let prev_offset = self.offset - 1;

        let mut valid = true;
        let mut n = 0;

        loop {
            let ch = self.cc;

            if ch == Some('\n') || ch.is_none() {
                if valid {
                    self.err("unterminated char literal", prev_offset);
                    valid = false
                }

                break;
            }

            self.next();

            if ch == Some('\'') {
                if n == 0 {
                    self.err("empty char literal", prev_offset);
                    valid = false;
                }

                break;
            }

            n += 1;

            if ch == Some('\\') {
                valid = self.scan_escape('\'');
            }
        }

        if valid && n != 1 {
            self.err("char literal too wide", prev_offset);
        }

        from_utf8(&self.src[prev_offset..self.offset]).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use crate::frontend::lexer::pos::FileSet;

    use super::*;
    use rand::*;

    struct TestCase {
        token: Tok,
        lit: &'static str,
    }

    #[derive(Clone)]
    struct LexerResult {
        lit: String,
        kind: Tok,
        line: usize,
        column: usize,
    }

    fn count_lines(s: &str) -> usize {
        if s.is_empty() {
            0
        } else {
            s.bytes().filter(|&b| b == b'\n').count() + 1
        }
    }

    fn test_lexer_result(
        set: &mut FileSet,
        input: &str,
        parse_comment: bool,
        expected: &[LexerResult],
    ) {
        let file = set.add_file("test_file".to_owned(), input.len());
        let src = input.as_bytes();

        let mut l = Lexer::new(file, src, parse_comment, |msg, pos| {
            eprintln!("Lexer error: {} at {:?}", msg, pos);
        });

        for (i, e) in expected.iter().enumerate() {
            let (tok, literal, offset) = l.scan();
            let src_pos = l.file.source_pos(offset);

            assert_eq!(
                tok, e.kind,
                "iteration ({}): expected token: {:?}, got: {:?}",
                i, e.kind, tok
            );
            assert_eq!(literal, e.lit, "iteration ({}): literal value not equal", i);
            assert_eq!(
                src_pos.line as usize, e.line,
                "iteration ({}): line number not synchronized",
                i
            );
            assert_eq!(
                src_pos.column as usize, e.column,
                "iteration ({}): column number not synchronized",
                i
            );
        }

        let (tok, _, _) = l.scan();
        assert_eq!(tok, Tok::Eof, "more tokens left in stream");
        assert_eq!(
            l.err_count, 0,
            "lexer reported errors during test execution"
        );
    }

    #[test]
    fn test_tokens() {
        let test_cases = vec![
            TestCase {
                token: Tok::Comment,
                lit: "# a comment\n",
            },
            TestCase {
                token: Tok::Comment,
                lit: "#\r\n",
            },
            TestCase {
                token: Tok::Identifier,
                lit: "foobar",
            },
            TestCase {
                token: Tok::Identifier,
                lit: "a۰۱۸",
            },
            TestCase {
                token: Tok::Identifier,
                lit: "foo६४",
            },
            TestCase {
                token: Tok::Identifier,
                lit: "bar９８７６",
            },
            TestCase {
                token: Tok::Identifier,
                lit: "ŝ",
            },
            TestCase {
                token: Tok::Identifier,
                lit: "ŝfoo",
            },
            TestCase {
                token: Tok::IntegerLiteral,
                lit: "0",
            },
            TestCase {
                token: Tok::IntegerLiteral,
                lit: "1",
            },
            TestCase {
                token: Tok::IntegerLiteral,
                lit: "123456789012345678890",
            },
            TestCase {
                token: Tok::IntegerLiteral,
                lit: "01234567",
            },
            TestCase {
                token: Tok::IntegerLiteral,
                lit: "0xcafebabe",
            },
            TestCase {
                token: Tok::RealLiteral,
                lit: "0.",
            },
            TestCase {
                token: Tok::RealLiteral,
                lit: ".0",
            },
            TestCase {
                token: Tok::RealLiteral,
                lit: "3.14159265",
            },
            TestCase {
                token: Tok::RealLiteral,
                lit: "1e0",
            },
            TestCase {
                token: Tok::RealLiteral,
                lit: "1e+100",
            },
            TestCase {
                token: Tok::RealLiteral,
                lit: "1e-100",
            },
            TestCase {
                token: Tok::RealLiteral,
                lit: "2.71828e-1000",
            },
            TestCase {
                token: Tok::CharLiteral,
                lit: "'a'",
            },
            TestCase {
                token: Tok::CharLiteral,
                lit: "'\\000'",
            },
            TestCase {
                token: Tok::CharLiteral,
                lit: "'\\xFF'",
            },
            TestCase {
                token: Tok::CharLiteral,
                lit: "'\\uff16'",
            },
            TestCase {
                token: Tok::CharLiteral,
                lit: "'\\U0000ff16'",
            },
            TestCase {
                token: Tok::Plus,
                lit: "+",
            },
            TestCase {
                token: Tok::Minus,
                lit: "-",
            },
            TestCase {
                token: Tok::Star,
                lit: "*",
            },
            TestCase {
                token: Tok::Slash,
                lit: "/",
            },
            TestCase {
                token: Tok::Percent,
                lit: "%",
            },
            TestCase {
                token: Tok::Ampersand,
                lit: "&",
            },
            TestCase {
                token: Tok::Pipe,
                lit: "|",
            },
            TestCase {
                token: Tok::Xor,
                lit: "^",
            },
            TestCase {
                token: Tok::LeftShift,
                lit: "<<",
            },
            TestCase {
                token: Tok::RightShift,
                lit: ">>",
            },
            TestCase {
                token: Tok::PlusEq,
                lit: "+=",
            },
            TestCase {
                token: Tok::MinusEq,
                lit: "-=",
            },
            TestCase {
                token: Tok::StarEq,
                lit: "*=",
            },
            TestCase {
                token: Tok::SlashEq,
                lit: "/=",
            },
            TestCase {
                token: Tok::PercentEq,
                lit: "%=",
            },
            TestCase {
                token: Tok::AndEq,
                lit: "&=",
            },
            TestCase {
                token: Tok::OrEq,
                lit: "|=",
            },
            TestCase {
                token: Tok::XorEq,
                lit: "^=",
            },
            TestCase {
                token: Tok::LshiftEq,
                lit: "<<=",
            },
            TestCase {
                token: Tok::RshiftEq,
                lit: ">>=",
            },
            TestCase {
                token: Tok::And,
                lit: "&&",
            },
            TestCase {
                token: Tok::Or,
                lit: "||",
            },
            TestCase {
                token: Tok::Equals,
                lit: "==",
            },
            TestCase {
                token: Tok::Less,
                lit: "<",
            },
            TestCase {
                token: Tok::Greater,
                lit: ">",
            },
            TestCase {
                token: Tok::Assign,
                lit: "=",
            },
            TestCase {
                token: Tok::Bang,
                lit: "!",
            },
            TestCase {
                token: Tok::BangEq,
                lit: "!=",
            },
            TestCase {
                token: Tok::LessEq,
                lit: "<=",
            },
            TestCase {
                token: Tok::GreaterEq,
                lit: ">=",
            },
            TestCase {
                token: Tok::Dot,
                lit: ".",
            },
            TestCase {
                token: Tok::DotDotDot,
                lit: "...",
            },
            TestCase {
                token: Tok::LeftParen,
                lit: "(",
            },
            TestCase {
                token: Tok::LeftBracket,
                lit: "[",
            },
            TestCase {
                token: Tok::LeftBrace,
                lit: "{",
            },
            TestCase {
                token: Tok::Comma,
                lit: ",",
            },
            TestCase {
                token: Tok::RightParen,
                lit: ")",
            },
            TestCase {
                token: Tok::RightBracket,
                lit: "]",
            },
            TestCase {
                token: Tok::RightBrace,
                lit: "}",
            },
            TestCase {
                token: Tok::Semicolon,
                lit: ";",
            },
            TestCase {
                token: Tok::Colon,
                lit: ":",
            },
            TestCase {
                token: Tok::Break,
                lit: "break",
            },
            TestCase {
                token: Tok::Continue,
                lit: "continue",
            },
        ];

        let mut rng = rng();
        let mut line_sum = 0;
        let mut lines = Vec::new();
        let mut set = FileSet::new();

        let mut line_numbers = vec![0; test_cases.len()];
        let mut column_numbers = vec![0; test_cases.len()];

        for (i, tc) in test_cases.iter().enumerate() {
            let empty_lines = rng.random_range(0..4);
            for _ in 0..empty_lines {
                lines.push(" ".repeat(rng.random_range(0..10)));
            }

            let empty_columns = rng.random_range(0..10);
            lines.push(format!(
                "{}{}{}",
                " ".repeat(empty_columns),
                tc.lit,
                " ".repeat(empty_columns)
            ));

            line_numbers[i] = line_sum + empty_lines + 1;
            line_sum += empty_lines + count_lines(tc.lit);
            column_numbers[i] = empty_columns + 1;
        }

        let mut expected = Vec::new();
        let mut expected_skip_comments = Vec::new();

        for (i, tc) in test_cases.iter().enumerate() {
            let expected_literal = match tc.token {
                Tok::Comment => &tc.lit[..tc.lit.len() - 1],
                _ => tc.lit,
            };

            let res = LexerResult {
                lit: expected_literal.to_owned(),
                kind: tc.token.clone(),
                line: line_numbers[i],
                column: column_numbers[i],
            };

            expected.push(res.clone());
            if tc.token != Tok::Comment {
                expected_skip_comments.push(res);
            }
        }

        let full_input = lines.join("\n");

        test_lexer_result(&mut set, &full_input, true, &expected);
        test_lexer_result(&mut set, &full_input, false, &expected_skip_comments);
    }
}

use crate::frontend::lexer::{
    pos::{LexFile, SourcePosition},
    token::{Tok, get_ident_or_keyword},
};
use std::str::from_utf8;

const EOF: char = '\0';
const BOM: char = '\u{FEFF}';

#[inline(always)]
fn utf8_width(b: u8) -> usize {
    match b {
        0x00..=0x7F => 1,
        0xC2..=0xDF => 2,
        0xE0..=0xEF => 3,
        0xF0..=0xF4 => 4,
        _ => 0,
    }
}

#[inline(always)]
fn hex_digit_value(ch: char) -> u32 {
    ch.to_digit(16).unwrap_or(16)
}

pub struct Lexer<'a, E>
where
    E: Fn(&str, &SourcePosition),
{
    file: &'a mut LexFile,
    src: &'a [u8],
    parse_comment: bool,
    err_cb: E,
    err_count: u64,
    cc: char,
    offset: usize,
    read_offset: usize,
}

impl<'a, E> Lexer<'a, E>
where
    E: Fn(&str, &SourcePosition),
{
    pub fn new(file: &'a mut LexFile, src: &'a [u8], parse_comment: bool, err_cb: E) -> Self {
        let mut lexer = Self {
            file,
            src,
            parse_comment,
            err_cb,
            err_count: 0,
            cc: ' ',
            offset: 0,
            read_offset: 0,
        };

        lexer.next();
        if lexer.cc == BOM {
            lexer.next(); // strip BOM
        }

        lexer
    }

    pub fn scan(&mut self) -> (Tok, String, u64) {
        let mut lit = "";
        let mut tok;

        self.scan_whitespace();

        let pos = self.file.base.saturating_add(self.offset as u64);

        let c = self.cc;

        if c == '_' || c.is_alphabetic() {
            lit = self.scan_ident();
            tok = get_ident_or_keyword(lit);
        } else if c.is_ascii_digit() || (c == '.' && self.peek().is_ascii_digit()) {
            (tok, lit) = self.scan_number();
        } else {
            self.next();

            match c {
                EOF => tok = Tok::Eof,

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
                    if self.cc == '=' {
                        tok = Tok::Equals;
                        self.next();
                    }
                }

                '+' => {
                    tok = Tok::Plus;
                    if self.cc == '=' {
                        tok = Tok::PlusEq;
                        self.next();
                    }
                }

                '*' => {
                    tok = Tok::Star;
                    if self.cc == '=' {
                        tok = Tok::StarEq;
                        self.next();
                    }
                }

                '/' => {
                    tok = Tok::Slash;
                    if self.cc == '=' {
                        tok = Tok::SlashEq;
                        self.next();
                    }
                }

                '%' => {
                    tok = Tok::Percent;
                    if self.cc == '=' {
                        tok = Tok::PercentEq;
                        self.next();
                    }
                }

                '!' => {
                    tok = Tok::Bang;
                    if self.cc == '=' {
                        tok = Tok::BangEq;
                        self.next();
                    }
                }

                '^' => {
                    tok = Tok::Xor;
                    if self.cc == '=' {
                        tok = Tok::XorEq;
                        self.next();
                    }
                }

                '-' => {
                    tok = Tok::Minus;
                    if self.cc == '=' {
                        tok = Tok::MinusEq;
                        self.next();
                    }
                }

                '>' => {
                    tok = Tok::Greater;
                    if self.cc == '>' {
                        tok = Tok::RightShift;
                        self.next();
                        if self.cc == '=' {
                            tok = Tok::RshiftEq;
                            self.next();
                        }
                    } else if self.cc == '=' {
                        tok = Tok::GreaterEq;
                        self.next();
                    }
                }

                '<' => {
                    tok = Tok::Less;
                    if self.cc == '<' {
                        tok = Tok::LeftShift;
                        self.next();
                        if self.cc == '=' {
                            tok = Tok::LshiftEq;
                            self.next();
                        }
                    } else if self.cc == '=' {
                        tok = Tok::LessEq;
                        self.next();
                    }
                }

                '&' => {
                    tok = Tok::Ampersand;
                    if self.cc == '=' {
                        tok = Tok::AndEq;
                        self.next();
                    } else if self.cc == '&' {
                        tok = Tok::And;
                        self.next();
                    }
                }

                '|' => {
                    tok = Tok::Pipe;
                    if self.cc == '=' {
                        tok = Tok::OrEq;
                        self.next();
                    } else if self.cc == '|' {
                        tok = Tok::Or;
                        self.next();
                    }
                }

                '.' => {
                    tok = Tok::Dot;
                    if self.cc == '.' {
                        self.next();
                        if self.cc == '.' {
                            tok = Tok::DotDotDot;
                            self.next();
                        } else {
                            self.err(
                                "invalid ellipsis mark",
                                self.file.lex_offset(pos).unwrap() as usize,
                            );
                        }
                    }
                }

                _ => {
                    if c != BOM {
                        self.err(
                            "unexpected unicode character",
                            self.file.lex_offset(pos).unwrap() as usize,
                        );
                    }

                    let lit = Tok::Invalid.to_str().to_owned();
                    return (Tok::Invalid, lit, pos);
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

    #[inline(always)]
    fn err(&mut self, msg: &str, offset: usize) {
        (self.err_cb)(msg, &self.file.source_pos(offset as u64));
        self.err_count += 1;
    }

    fn next(&mut self) {
        if self.read_offset < self.src.len() {
            self.offset = self.read_offset;

            if self.cc == '\n' {
                self.file.add_line(self.offset as u64);
            }

            let b = self.src[self.read_offset];

            let (ch, width) = match b {
                0 => {
                    self.err("unexpected null character", self.offset);
                    ('\0', 1)
                }

                0x00..=0x7F => (b as char, 1),

                _ => {
                    let w = utf8_width(b);

                    if w == 0 || self.read_offset + w > self.src.len() {
                        self.err("unexpected unicode character", self.offset);
                        (char::REPLACEMENT_CHARACTER, 1)
                    } else {
                        match from_utf8(&self.src[self.read_offset..self.read_offset + w]) {
                            Ok(s) => {
                                let ch = s.chars().next().unwrap();

                                if ch == BOM && self.offset > 0 {
                                    self.err("unexpected BOM encounter", self.offset);
                                }

                                (ch, w)
                            }

                            Err(_) => {
                                self.err("unexpected unicode character", self.offset);
                                (char::REPLACEMENT_CHARACTER, w)
                            }
                        }
                    }
                }
            };

            self.read_offset += width;
            self.cc = ch;
        } else {
            self.offset = self.src.len();
            if self.cc == '\n' {
                self.file.add_line(self.offset as u64);
            }

            self.cc = EOF;
        }
    }

    #[inline(always)]
    fn peek(&self) -> u8 {
        if self.read_offset < self.src.len() {
            return self.src[self.read_offset];
        }

        0
    }

    #[inline]
    fn scan_ident(&mut self) -> &'a str {
        let prev_offset = self.offset;
        while self.cc.is_alphabetic() || self.cc.is_numeric() {
            self.next();
        }

        from_utf8(&self.src[prev_offset..self.offset]).unwrap()
    }

    #[inline]
    fn scan_whitespace(&mut self) {
        while self.cc.is_whitespace() {
            self.next();
        }
    }

    #[inline]
    fn scan_comment(&mut self) -> &'a str {
        let prev_offset = self.offset - 1;
        while self.cc != '\n' && self.cc >= 0 as char {
            self.next()
        }

        from_utf8(&self.src[prev_offset..self.offset]).unwrap()
    }

    #[inline]
    fn scan_digit_seq(&mut self, base: u32) {
        while self.cc == '_' || hex_digit_value(self.cc) < base {
            self.next()
        }
    }

    fn scan_number(&mut self) -> (Tok, &'a str) {
        let mut tok = Tok::IntegerLiteral;
        let prev_offset = self.offset;
        let mut base = 10;

        match (self.cc, self.peek().to_ascii_lowercase() as char) {
            ('0', 'b') => {
                base = 2;
                self.next();
                self.next();
            }

            ('0', 'o') => {
                base = 8;
                self.next();
                self.next();
            }

            ('0', 'x') => {
                base = 16;
                self.next();
                self.next();
            }

            _ => {}
        }

        if base != 10 && hex_digit_value(self.peek() as char) == 16 {
            self.err("no digits after base specifier", prev_offset);
            return (tok, from_utf8(&self.src[prev_offset..self.offset]).unwrap());
        }

        // scan whole number
        self.scan_digit_seq(base);

        // scan fractional
        if self.cc == '.' && base == 10 {
            tok = Tok::RealLiteral;
            self.next();
            self.scan_digit_seq(base)
        }

        // scan exponent
        if self.cc == 'e' || self.cc == 'E' {
            tok = Tok::RealLiteral;
            self.next();

            // scan exponent sign
            if self.cc == '-' || self.cc == '+' {
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

        match self.cc {
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
                let msg = if self.cc == EOF {
                    "unterminated escape sequence"
                } else {
                    "unknown escape sequence"
                };

                self.err(msg, prev_offset);
                return false;
            }
        }

        let mut x = 0;

        while n > 0 {
            let d = hex_digit_value(self.cc);

            if d >= base {
                let mut msg = "illegal unicode escape sequence";

                if self.cc == EOF {
                    msg = "unterminated escape sequence";
                }

                self.err(msg, self.offset);
                return false;
            }

            x = x * base + d;
            self.next();
            n -= 1;
        }

        if x > max || 0xD800 <= x && x < 0xE000 {
            self.err("illegal unicode escape sequence", prev_offset);
            return false;
        }

        true
    }

    fn scan_string(&mut self) -> &'a str {
        let prev_offset = self.offset - 1;
        loop {
            let ch = self.cc;

            if ch == '\n' || ch < 0 as char {
                self.err("unterminated string literal", prev_offset);
                break;
            }

            self.next();

            if ch == '"' {
                break;
            }

            if ch == '\\' {
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

            if ch == '\n' || ch == EOF {
                if valid {
                    self.err("unterminated char literal", prev_offset);
                    valid = false
                }

                break;
            }

            self.next();

            if ch == '\'' {
                if n == 0 {
                    self.err("empty char literal", prev_offset);
                    valid = false;
                }

                break;
            }

            n += 1;

            if ch == '\\' {
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
        let file = set.add_file("test_file".to_owned(), input.len() as u64);
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

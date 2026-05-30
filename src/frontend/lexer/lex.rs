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
    do_asi: bool,
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
    pub fn new(
        file: &'a mut LexFile,
        src: &'a [u8],
        parse_comment: bool,
        do_asi: bool,
        err_cb: E,
    ) -> Self {
        let mut lexer = Self {
            file,
            src,
            parse_comment,
            do_asi,
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

        let pos = self.file.get_file_base() + self.offset as u64;

        let mut asi = false;
        let c = self.cc;

        if c == '_' || c.is_alphabetic() {
            lit = self.scan_ident();
            tok = get_ident_or_keyword(lit);
            asi = tok.needs_asi();
        } else if c.is_ascii_digit() || (c == '.' && self.peek().is_ascii_digit()) {
            asi = true;
            (tok, lit) = self.scan_number();
        } else {
            self.next();

            match c {
                EOF => {
                    if self.do_asi {
                        self.do_asi = false;
                        return (Tok::Semicolon, "\n".to_owned(), pos);
                    }

                    tok = Tok::Eof;
                }

                '\n' => {
                    self.do_asi = false;
                    return (Tok::Semicolon, "\n".to_owned(), pos);
                }

                '"' => {
                    asi = true;
                    tok = Tok::StringLiteral;
                    lit = self.scan_string();
                }

                '\'' => {
                    asi = true;
                    tok = Tok::CharLiteral;
                    lit = self.scan_char();
                }

                '#' => {
                    if self.do_asi {
                        self.cc = '#';
                        self.offset = self.file.lex_offset(pos).unwrap() as usize;
                        self.read_offset = self.offset + 1;
                        self.do_asi = false;
                        return (Tok::Semicolon, "\n".to_owned(), pos);
                    }

                    let comment = self.scan_comment();
                    if !self.parse_comment {
                        self.do_asi = false;
                        return self.scan();
                    }

                    tok = Tok::Comment;
                    lit = comment;
                }

                '?' => tok = Tok::Question,

                '(' => tok = Tok::LeftParen,
                ')' => {
                    tok = Tok::RightParen;
                    asi = true;
                }

                '[' => tok = Tok::LeftBracket,
                ']' => {
                    tok = Tok::RightBracket;
                    asi = true;
                }

                '{' => tok = Tok::LeftBrace,
                '}' => {
                    tok = Tok::RightBrace;
                    asi = true;
                }

                ',' => tok = Tok::Comma,
                ';' => {
                    tok = Tok::Semicolon;
                    lit = ";";
                }

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

                    self.do_asi = asi;
                    let lit = Tok::Invalid.to_str().to_owned();
                    return (Tok::Invalid, lit, pos);
                }
            }
        }

        self.do_asi = asi;
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
        while self.cc == ' '
            || self.cc == '\t'
            || self.cc == '\r'
            || (self.cc == '\n' && !self.do_asi)
        {
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
        let mut prev_offset = self.offset;
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

            prev_offset = self.offset;
            self.scan_digit_seq(10);

            if prev_offset == self.offset {
                self.err("no digits after exponent", prev_offset);
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

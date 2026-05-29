use crate::frontend::lexer::{
    pos::{LexFile, SourcePosition},
    token::Tok,
};
use std::str::from_utf8;

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
    eof: bool,
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
            eof: false,
        };

        lexer.next();
        if lexer.cc == '\u{FEFF}' {
            lexer.next(); // strip BOM
        }

        lexer
    }

    pub fn scan(&mut self) -> (Tok, String, u64) {
        let mut literal = "";
        let mut tok = Tok::Invalid;

        self.scan_whitespace();

        let pos = self.file.get_file_base() + self.offset as u64;

        (tok, literal.to_owned(), pos)
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

                                if ch == '\u{FEFF}' && self.offset > 0 {
                                    self.err("unexpected BOM encounter", self.offset);
                                }

                                (ch, w)
                            }

                            Err(_) => {
                                self.err("unexpected unicode character", self.offset);
                                (char::REPLACEMENT_CHARACTER, 1)
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

            self.eof = true
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
    fn scan_ident(&mut self) -> &str {
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
    fn scan_comment(&mut self) -> &str {
        let prev_offset = self.offset - 1; // '#' already consumed
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

    fn scan_number(&mut self) -> (Tok, &str) {
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
}

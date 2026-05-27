use crate::frontend::lexer::{pos::LexFile, pos::SourcePosition};

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

            let mut width = 1 as usize;
            let b = self.src[self.read_offset];

            let ch = match b {
                0 => {
                    self.err("unexpected null character", self.offset);
                    '\0'
                }

                0x00..=0x7F => b as char,

                _ => match std::str::from_utf8(&self.src[self.read_offset..]) {
                    Ok(s) => {
                        let ch = s.chars().next().unwrap_or(char::REPLACEMENT_CHARACTER);
                        width = ch.len_utf8();

                        if ch == char::REPLACEMENT_CHARACTER && width == 1 {
                            self.err("unexpected unicode character", self.offset);
                        } else if ch == '\u{FEFF}' && self.offset > 0 {
                            self.err("unexpected BOM encounter", self.offset);
                        }

                        ch
                    }
                    Err(_) => {
                        self.err("unexpected unicode character", self.offset);
                        char::REPLACEMENT_CHARACTER
                    }
                },
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
}

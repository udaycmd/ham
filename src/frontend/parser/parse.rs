use crate::frontend::lexer::{
    lex::Lexer,
    pos::{File, Position, SourcePosition},
    token::Tok,
};

pub struct ParseError {
    pos: SourcePosition,
    message: String,
}

pub struct Parser<'a, E>
where
    E: Fn(String, SourcePosition),
{
    file: &'a mut File,
    lexer: Lexer<'a, E>,
    pos: Position,
    kind: Tok,
    literal: String,
    errors: Vec<ParseError>,
    max_report_errors: usize,
}

impl<'a, E> Parser<'a, E>
where
    E: Fn(String, SourcePosition),
{
    pub fn new(
        file: &'a mut File,
        src: &'a [u8],
        max_report_errors: usize,
        parse_comment: bool,
    ) -> Self {
    }
}

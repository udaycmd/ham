use std::fmt;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum Tok {
    Eof,
    Invalid,
    Comment,

    // - Keywords -
    Break,
    Continue,
    If,
    Else,
    Switch,
    Case,
    Default,
    Enum,
    Proc,
    For,
    Until,
    In,
    Import,
    Export,
    Return,
    Concept,
    Var,
    Const,
    True,
    False,
    Empty,
    Do,
    End,

    // - Punctuations -
    LeftParen,
    RightParen,
    LeftBracket,
    RightBracket,
    LeftBrace,
    RightBrace,
    Comma,
    Colon,
    Dot,
    DotDotDot,

    // - Operators -
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Ampersand,
    Pipe,
    Xor,
    LeftShift,
    RightShift,
    PlusEq,
    MinusEq,
    StarEq,
    SlashEq,
    PercentEq,
    XorEq,
    AndEq,
    OrEq,
    LshiftEq,
    RshiftEq,
    Assign,
    Equals,
    And,
    Or,
    Bang,
    BangEq,
    Less,
    Greater,
    LessEq,
    GreaterEq,
    Question,

    // - Literals -
    Identifier(String),
    StringLiteral(String),
    CharLiteral(char),
    IntegerLiteral(i64),
    UIntegerLiteral(u64),
    RealLiteral(f64),
}

impl fmt::Display for Tok {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Tok::Eof => "<eof>",
            Tok::Invalid => "<invalid-token>",
            Tok::Comment => "<comment>",

            Tok::Break => "break",
            Tok::Continue => "continue",
            Tok::If => "if",
            Tok::Else => "else",
            Tok::Switch => "switch",
            Tok::Case => "case",
            Tok::Default => "default",
            Tok::Enum => "enum",
            Tok::Proc => "proc",
            Tok::For => "for",
            Tok::Until => "until",
            Tok::In => "in",
            Tok::Import => "import",
            Tok::Export => "export",
            Tok::Return => "return",
            Tok::Concept => "concept",
            Tok::Var => "var",
            Tok::Const => "const",
            Tok::True => "true",
            Tok::False => "false",
            Tok::Empty => "empty",
            Tok::Do => "do",
            Tok::End => "end",

            Tok::LeftParen => "(",
            Tok::RightParen => ")",
            Tok::LeftBracket => "[",
            Tok::RightBracket => "]",
            Tok::LeftBrace => "{",
            Tok::RightBrace => "}",
            Tok::Comma => ",",
            Tok::Colon => ":",
            Tok::Dot => ".",
            Tok::DotDotDot => "...",

            Tok::Plus => "+",
            Tok::Minus => "-",
            Tok::Star => "*",
            Tok::Slash => "/",
            Tok::Percent => "%",
            Tok::Ampersand => "&",
            Tok::Pipe => "|",
            Tok::Xor => "^",
            Tok::LeftShift => "<<",
            Tok::RightShift => ">>",
            Tok::PlusEq => "+=",
            Tok::MinusEq => "-=",
            Tok::StarEq => "*=",
            Tok::XorEq => "^=",
            Tok::SlashEq => "/=",
            Tok::PercentEq => "%=",
            Tok::AndEq => "&=",
            Tok::OrEq => "|=",
            Tok::LshiftEq => "<<=",
            Tok::RshiftEq => ">>=",
            Tok::Equals => "==",
            Tok::Assign => "=",
            Tok::And => "&&",
            Tok::Or => "||",
            Tok::Bang => "!",
            Tok::BangEq => "!=",
            Tok::Less => "<",
            Tok::Greater => ">",
            Tok::LessEq => "<=",
            Tok::GreaterEq => ">=",
            Tok::Question => "?",

            Tok::Identifier(s) | Tok::StringLiteral(s) => return write!(f, "ident({})", s),
            Tok::CharLiteral(c) => return write!(f, "char({})", c),
            Tok::IntegerLiteral(i) => return write!(f, "i64({})", i),
            Tok::UIntegerLiteral(u) => return write!(f, "u64({})", u),
            Tok::RealLiteral(r) => return write!(f, "f64({})", r),
        };

        write!(f, "{s}")
    }
}

impl Tok {
    pub fn is_eof(&self) -> bool {
        matches!(self, Tok::Eof)
    }

    pub fn is_keyword(&self) -> bool {
        self >= &Tok::Break && self <= &Tok::End
    }

    pub fn is_punctuator(&self) -> bool {
        self >= &Tok::LeftParen && self <= &Tok::DotDotDot
    }

    pub fn is_operator(&self) -> bool {
        self >= &Tok::Plus && self <= &Tok::Question
    }

    pub fn prec(&self) -> i32 {
        match self {
            Tok::Equals | Tok::BangEq => 1,
            Tok::Less | Tok::LessEq | Tok::Greater | Tok::GreaterEq => 2,
            Tok::And
            | Tok::Or
            | Tok::RightShift
            | Tok::LeftShift
            | Tok::Xor
            | Tok::Ampersand
            | Tok::Pipe => 3,
            Tok::Plus | Tok::Minus => 4,
            Tok::Star | Tok::Slash | Tok::Percent => 5,
            _ => 0,
        }
    }
}

pub fn get_ident_or_keyword(s: &str) -> Tok {
    match s {
        "break" => Tok::Break,
        "continue" => Tok::Continue,
        "if" => Tok::If,
        "else" => Tok::Else,
        "switch" => Tok::Switch,
        "case" => Tok::Case,
        "default" => Tok::Default,
        "enum" => Tok::Enum,
        "proc" => Tok::Proc,
        "for" => Tok::For,
        "until" => Tok::Until,
        "in" => Tok::In,
        "import" => Tok::Import,
        "export" => Tok::Export,
        "return" => Tok::Return,
        "concept" => Tok::Concept,
        "var" => Tok::Var,
        "const" => Tok::Const,
        "true" => Tok::True,
        "false" => Tok::False,
        "empty" => Tok::Empty,
        "do" => Tok::Do,
        "end" => Tok::End,
        ident => Tok::Identifier(ident.to_owned()),
    }
}

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
        match self {
            Tok::Eof => write!(f, "<eof>"),
            Tok::Invalid => write!(f, "<invalid-token>"),
            Tok::Comment => write!(f, "<comment>"),

            Tok::Break => write!(f, "break"),
            Tok::Continue => write!(f, "continue"),
            Tok::If => write!(f, "if"),
            Tok::Else => write!(f, "else"),
            Tok::Switch => write!(f, "switch"),
            Tok::Case => write!(f, "case"),
            Tok::Default => write!(f, "default"),
            Tok::Enum => write!(f, "enum"),
            Tok::Proc => write!(f, "proc"),
            Tok::For => write!(f, "for"),
            Tok::Until => write!(f, "until"),
            Tok::In => write!(f, "in"),
            Tok::Import => write!(f, "import"),
            Tok::Export => write!(f, "export"),
            Tok::Return => write!(f, "return"),
            Tok::Concept => write!(f, "concept"),
            Tok::Var => write!(f, "var"),
            Tok::Const => write!(f, "const"),
            Tok::True => write!(f, "true"),
            Tok::False => write!(f, "false"),
            Tok::Empty => write!(f, "empty"),
            Tok::Do => write!(f, "do"),
            Tok::End => write!(f, "end"),

            Tok::LeftParen => write!(f, "("),
            Tok::RightParen => write!(f, ")"),
            Tok::LeftBracket => write!(f, "["),
            Tok::RightBracket => write!(f, "]"),
            Tok::LeftBrace => write!(f, "{{"),
            Tok::RightBrace => write!(f, "}}"),
            Tok::Comma => write!(f, ","),
            Tok::Colon => write!(f, ":"),
            Tok::Dot => write!(f, "."),
            Tok::DotDotDot => write!(f, "..."),

            Tok::Plus => write!(f, "+"),
            Tok::Minus => write!(f, "-"),
            Tok::Star => write!(f, "*"),
            Tok::Slash => write!(f, "/"),
            Tok::Percent => write!(f, "%"),
            Tok::Ampersand => write!(f, "&"),
            Tok::Pipe => write!(f, "|"),
            Tok::Xor => write!(f, "^"),
            Tok::LeftShift => write!(f, "<<"),
            Tok::RightShift => write!(f, ">>"),
            Tok::PlusEq => write!(f, "+="),
            Tok::MinusEq => write!(f, "-="),
            Tok::StarEq => write!(f, "*="),
            Tok::XorEq => write!(f, "^="),
            Tok::SlashEq => write!(f, "/="),
            Tok::PercentEq => write!(f, "%="),
            Tok::AndEq => write!(f, "&="),
            Tok::OrEq => write!(f, "|="),
            Tok::LshiftEq => write!(f, "<<="),
            Tok::RshiftEq => write!(f, ">>="),
            Tok::Equals => write!(f, "=="),
            Tok::Assign => write!(f, "="),
            Tok::And => write!(f, "&&"),
            Tok::Or => write!(f, "||"),
            Tok::Bang => write!(f, "!"),
            Tok::BangEq => write!(f, "!="),
            Tok::Less => write!(f, "<"),
            Tok::Greater => write!(f, ">"),
            Tok::LessEq => write!(f, "<="),
            Tok::GreaterEq => write!(f, ">="),
            Tok::Question => write!(f, "?"),

            Tok::Identifier(s) | Tok::StringLiteral(s) => write!(f, "ident({})", s),
            Tok::CharLiteral(c) => write!(f, "char({})", c),
            Tok::IntegerLiteral(i) => write!(f, "i64({})", i),
            Tok::UIntegerLiteral(u) => write!(f, "u64({})", u),
            Tok::RealLiteral(r) => write!(f, "f64({})", r),
        }
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

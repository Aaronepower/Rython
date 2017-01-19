#[derive(Clone, Debug)]
pub enum Lexeme<'a> {
    Bytes(usize, Vec<u8>),
    Dedent,
    Delimiter(usize, Delimiter),
    Float(f64),
    Identifier(usize, &'a str),
    Indent,
    Integer(i64),
    Keyword(usize, Keyword),
    Newline,
    Operator(Operator),
    Str(usize, String),
}

impl<'a> Lexeme<'a> {
    pub fn is_unary(&self) -> bool {
        match *self {
            Lexeme::Operator(Operator::UnaryAdd) |
            Lexeme::Operator(Operator::UnarySub) |
            Lexeme::Operator(Operator::UnaryNot) => true,
            _ => false,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Delimiter {
    DictClose,
    DictOpen,
    ListClose,
    ListOpen,
    ParenClose,
    ParenOpen,
}

impl Delimiter {

    pub fn is_delimiter(ch: char) -> Option<Self> {
        use self::Delimiter::*;

        match ch {
            '{' => Some(DictOpen),
            '}' => Some(DictClose),
            '[' => Some(ListOpen),
            ']' => Some(ListClose),
            '(' => Some(ParenOpen),
            ')' => Some(ParenClose),
            _ => None,
        }
    }

    pub fn is_closing(&self) -> bool {
        !self.is_opening()
    }

    pub fn is_opening(&self) -> bool {
        use self::Delimiter::*;

        match *self {
            DictOpen | ListOpen | ParenOpen => true,
            _ => false,
        }
    }

    pub fn is_matching(&self, delimiter: Self) -> bool {
        use self::Delimiter::*;

        delimiter == match *self {
            DictClose => DictOpen,
            ListClose => ListOpen,
            ParenClose => ParenOpen,
            DictOpen => DictClose,
            ListOpen => ListClose,
            ParenOpen => ParenClose,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Operator {
    Access,
    Add,
    AddAssign,
    And,
    AndAssign,
    Colon,
    Assign,
    Dec,
    DecAssign,
    Div,
    DivAssign,
    Equals,
    FuncAnno,
    LessThan,
    LessThanEqual,
    MoreThan,
    MoreThanEqual,
    Mul,
    MulAssign,
    Not,
    NotEquals,
    Or,
    OrAssign,
    Pow,
    PowAssign,
    Rem,
    RemAssign,
    FloorDiv,
    FloorDivAssign,
    Sep,
    Shl,
    ShlAssign,
    Shr,
    ShrAssign,
    Sub,
    SubAssign,
    Term,
    UnaryAdd,
    UnarySub,
    UnaryNot,
    Xor,
    XorAssign,
}

impl Operator {
    pub fn is_operator_term(ch: char) -> bool {
        match ch {
            '+' | '=' | '&' | '@' | '/' |
            '<' | '>' | '*' | '~' | '!' |
            '|' | '%' | '^' | '-' | ',' |
            ':' | ';' | '.' => true,
            _ => false,
        }
    }
    pub fn is_operator(string: &str) -> Option<Self> {
        use self::Operator::*;
        match string {
            "." => Some(Access),
            "+" => Some(Add),
            "+=" => Some(AddAssign),
            "&" => Some(And),
            "&=" => Some(AndAssign),
            ":" => Some(Colon),
            "=" => Some(Assign),
            "@" => Some(Dec),
            "@=" => Some(DecAssign),
            "," => Some(Sep),
            "/" => Some(Div),
            "/=" => Some(DivAssign),
            "==" => Some(Equals),
            "->" => Some(FuncAnno),
            "<" => Some(LessThan),
            "<=" => Some(LessThanEqual),
            ">" => Some(MoreThan),
            ">=" => Some(MoreThanEqual),
            "*" => Some(Mul),
            "*=" => Some(MulAssign),
            "~" => Some(Not),
            "!=" => Some(NotEquals),
            "|" => Some(Or),
            "|=" => Some(OrAssign),
            "**" => Some(Pow),
            "**=" => Some(PowAssign),
            "%" => Some(Rem),
            "%=" => Some(RemAssign),
            "//" => Some(FloorDiv),
            "//=" => Some(FloorDivAssign),
            "<<" => Some(Shl),
            "<<=" => Some(ShlAssign),
            ">>" => Some(Shr),
            ">>=" => Some(ShrAssign),
            "-" => Some(Sub),
            "-=" => Some(SubAssign),
            ";" => Some(Term),
            "^" => Some(Xor),
            "^=" => Some(XorAssign),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Prefix {
    Raw,
    Formatted,
    Bytes,
    Ignore,
}

impl Prefix {
    pub fn is_prefix(word: &str) -> Option<[Self; 2]> {
        use self::Prefix::*;
        match word {
            "B" => Some([Bytes, Ignore]),
            "BR" => Some([Bytes, Raw]),
            "Br" => Some([Bytes, Raw]),
            "F" => Some([Formatted, Ignore]),
            "FR" => Some([Formatted, Raw]),
            "Fr" => Some([Formatted, Raw]),
            "R" => Some([Raw, Ignore]),
            "RB" => Some([Bytes, Raw]),
            "RF" => Some([Formatted, Raw]),
            "Rb" => Some([Bytes, Raw]),
            "Rf" => Some([Formatted, Raw]),
            "U" => Some([Ignore, Ignore]),
            "b" => Some([Bytes, Ignore]),
            "bR" => Some([Bytes, Raw]),
            "br" => Some([Bytes, Raw]),
            "f" => Some([Formatted, Ignore]),
            "fR" => Some([Formatted, Raw]),
            "fr" => Some([Formatted, Raw]),
            "r" => Some([Raw, Ignore]),
            "rB" => Some([Bytes, Raw]),
            "rF" => Some([Formatted, Raw]),
            "rb" => Some([Bytes, Raw]),
            "rf" => Some([Formatted, Raw]),
            "u" => Some([Ignore, Ignore]),
            _ => None,
        }
    }
}


#[derive(Clone, Debug)]
pub enum Keyword {
    And,
    As,
    Assert,
    Await,
    Break,
    Class,
    Continue,
    Def,
    Del,
    Elif,
    Else,
    Except,
    False,
    Finally,
    For,
    From,
    Global,
    If,
    Import,
    In,
    Is,
    Lambda,
    NonLocal,
    None,
    Not,
    Or,
    Pass,
    Raise,
    Return,
    True,
    Try,
    While,
    With,
    Yield,
}

impl Keyword {
    pub fn is_keyword(name: &str) -> Option<Self> {
        match name {
            "False" => Some(Keyword::False),
            "None" => Some(Keyword::None),
            "True" => Some(Keyword::True),
            "and" => Some(Keyword::And),
            "as" => Some(Keyword::As),
            "assert" => Some(Keyword::Assert),
            "await" => Some(Keyword::Await),
            "break" => Some(Keyword::Break),
            "class" => Some(Keyword::Class),
            "continue" => Some(Keyword::Continue),
            "def" => Some(Keyword::Def),
            "del" => Some(Keyword::Del),
            "elif" => Some(Keyword::Elif),
            "else" => Some(Keyword::Else),
            "except" => Some(Keyword::Except),
            "finally" => Some(Keyword::Finally),
            "for" => Some(Keyword::For),
            "from" => Some(Keyword::From),
            "global" => Some(Keyword::Global),
            "if" => Some(Keyword::If),
            "import" => Some(Keyword::Import),
            "in" => Some(Keyword::In),
            "is" => Some(Keyword::Is),
            "lambda" => Some(Keyword::Lambda),
            "nonlocal" => Some(Keyword::NonLocal),
            "not" => Some(Keyword::Not),
            "or" => Some(Keyword::Or),
            "pass" => Some(Keyword::Pass),
            "raise" => Some(Keyword::Raise),
            "return" => Some(Keyword::Return),
            "try" => Some(Keyword::Try),
            "while" => Some(Keyword::While),
            "with" => Some(Keyword::With),
            "yield" => Some(Keyword::Yield),
            _ => None,
        }
    }
}

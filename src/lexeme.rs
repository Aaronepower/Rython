#[derive(Clone, Debug, Default)]
pub struct PhysicalLine<'a>(pub Vec<Lexeme<'a>>);

impl<'a> PhysicalLine<'a> {
    pub fn is_empty(&self) -> bool {
        let PhysicalLine(ref vec) = *self;
        vec.is_empty()
    }
}

#[derive(Clone, Debug)]
pub enum Lexeme<'a> {
    Bytes(usize, Vec<u8>),
    Dedent,
    Delimiter(usize, Delimiter),
    Identifier(usize, &'a str),
    Float(f64),
    Indent,
    Integer(i64),
    Keyword(usize, Keyword),
    Operator(Operator),
    Str(usize, String),
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

#[derive(Clone, Debug)]
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
    RoundDiv,
    RoundDivAssign,
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
            "//" => Some(RoundDiv),
            "//=" => Some(RoundDivAssign),
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
    False,
    Class,
    Finally,
    Is,
    Return,
    None,
    Continue,
    For,
    Lambda,
    Try,
    True,
    Def,
    From,
    NonLocal,
    While,
    And,
    Del,
    Global,
    Not,
    With,
    As,
    Elif,
    If,
    Or,
    Yield,
    Assert,
    Else,
    Import,
    Pass,
    Break,
    Except,
    In,
    Raise,
}

impl Keyword {
    pub fn is_keyword(name: &str) -> Option<Self> {
        match name {
            "False" => Some(Keyword::False),
            "class" => Some(Keyword::Class),
            "finally" => Some(Keyword::Finally),
            "is" => Some(Keyword::Is),
            "return" => Some(Keyword::Return),
            "None" => Some(Keyword::None),
            "continue" => Some(Keyword::Continue),
            "for" => Some(Keyword::For),
            "lambda" => Some(Keyword::Lambda),
            "try" => Some(Keyword::Try),
            "True" => Some(Keyword::True),
            "def" => Some(Keyword::Def),
            "from" => Some(Keyword::From),
            "nonlocal" => Some(Keyword::NonLocal),
            "while" => Some(Keyword::While),
            "and" => Some(Keyword::And),
            "del" => Some(Keyword::Del),
            "global" => Some(Keyword::Global),
            "not" => Some(Keyword::Not),
            "with" => Some(Keyword::With),
            "as" => Some(Keyword::As),
            "elif" => Some(Keyword::Elif),
            "if" => Some(Keyword::If),
            "or" => Some(Keyword::Or),
            "yield" => Some(Keyword::Yield),
            "assert" => Some(Keyword::Assert),
            "else" => Some(Keyword::Else),
            "import" => Some(Keyword::Import),
            "pass" => Some(Keyword::Pass),
            "break" => Some(Keyword::Break),
            "except" => Some(Keyword::Except),
            "in" => Some(Keyword::In),
            "raise" => Some(Keyword::Raise),
            _ => None,
        }
    }
}

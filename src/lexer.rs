use std::char;
use std::fmt;
use std::ops;
use std::str::CharIndices;
use itertools::multipeek;
use itertools::structs::MultiPeek;

pub type LexResult<'a> = Result<Lexeme<'a>, LexerError<'a>>;

use symbols::*;
use lexeme::*;
use self::LexerError::*;

macro_rules! get_or_eof {
    ($next:expr) => {
        match $next {
            Some(tuple) => tuple,
            None => return Err(Eof),
        }
    }
}

pub struct Lexer<'a> {
    iter: MultiPeek<CharIndices<'a>>,
    output: Vec<Lexeme<'a>>,
    stack: Vec<u8>,
    source: &'a str,
}

impl<'a> Lexer<'a> {

    pub fn new(source: &'a str) -> Self {
        Lexer {
            iter: multipeek(source.char_indices()),
            output: Vec::new(),
            stack: Vec::new(),
            source: source,
        }
    }

    pub fn lex(&mut self) {
        loop {
            match self.lex_line() {
                Err(LexerError::Continue) => continue,
                Err(LexerError::Eof) => break,
                Err(error) => panic!("{:?}", error),
                Ok((indent, lex)) => {
                    self.output.extend(lex);
                    self.output.push(Lexeme::Newline);
                    self.stack.push(indent);
                },
            }
        }
    }

    pub fn output(self) -> Vec<Lexeme<'a>> {
        self.output
    }

    fn lex_line(&mut self) -> Result<(u8, Vec<Lexeme<'a>>), LexerError<'a>> {
        let mut line = Vec::new();
        let mut indent = 0;
        let mut delimit_stack = Vec::new();

        self.reset_peek();

        while let Some(&(_, ch)) = self.peek() {
            match ch {
                ' ' | '\t' => {
                    indent += 1;
                    self.consume();
                }
                NEWLINE | FORMFEED | CARRIAGE => {
                    self.consume();
                    return Err(Continue);
                }
                _ => break,
            }
        }

        for &last in self.stack.iter().rev() {
            if indent == last {
                break;
            } else if indent > last {
                line.push(Lexeme::Indent);
                break;
            } else {
                line.push(Lexeme::Dedent);
            }
        }

        'line: loop {
            let (start, ch) = get_or_eof!(self.next());

            if ch.is_alphabetic() || ch == '_' {

                line.push(self.lex_word(start)?);
            } else if ch.is_digit(10) {

                line.push(self.lex_number(ch, start)?);
            } else if ch == '.' {

                line.push(self.lex_leading_dot(start)?);
            } else if ch == '\'' || ch == '"' {

                const NO_PREFIX: [Prefix; 2] = [Prefix::Ignore, Prefix::Ignore];
                line.push(self.lex_str(start, ch, NO_PREFIX)?);
            } else if Operator::is_operator_term(ch) {

                line.push(self.lex_operator(start)?);
            } else if let Some(delimiter) = Delimiter::is_delimiter(ch) {

                line.push(self.lex_delimiter(start, delimiter, &mut delimit_stack));
            } else if ch == '#' {

                loop {
                    let &(_, ch) = get_or_eof!(self.peek());

                    if ch == NEWLINE || ch == CARRIAGE || ch == FORMFEED {
                        continue 'line;
                    } else {
                        self.consume();
                    }
                }
            } else if ch == ESCAPE {

                self.lex_escape()?;
            } else if ch == NEWLINE || ch == CARRIAGE || ch == FORMFEED {

                if delimit_stack.is_empty() {
                    break 'line;
                }
            }
        }
        Ok((indent, line))
    }

    fn consume(&mut self) {
        let _ = self.next();
    }

    fn lex_leading_dot(&mut self, start: usize) -> LexResult<'a> {
        let mut string = String::from(".");

        loop {
            if let Some(&(_, ch)) = self.peek() {
                if !ch.is_digit(10) && ch != 'e' && ch != '+' && ch != '-' {
                    break;
                }
                self.consume();
                string.push(ch);
            } else {
                break;
            }
        }

        if string == "." {
            Ok(Lexeme::Operator(Operator::Access))
        } else {
            Ok(self.lex_float(&string, start)?)
        }
    }

    fn lex_delimiter(&mut self,
                     start: usize,
                     delimiter: Delimiter,
                     delimit_stack: &mut Vec<Delimiter>)
        -> Lexeme<'a>
    {
        let mut pop = false;

        if delimiter.is_opening() {
            delimit_stack.push(delimiter);
        } else if let Some(opening) = delimit_stack.last() {
            if opening.is_matching(delimiter) {
                pop = true;
            }
        }

        if pop {
            let _ = delimit_stack.pop();
        }

        Lexeme::Delimiter(start, delimiter)
    }

    fn lex_number(&mut self, ch: char, start: usize)
        -> LexResult<'a>
    {
        if ch == '0' {
            if let Some(&(_, ch)) = self.peek() {
                if !ch.is_whitespace() && !Operator::is_operator_term(ch) {
                    self.consume();
                    let number = match ch {
                        'b' | 'B' => self.lex_binary(),
                        'o' | 'O' => self.lex_octal(),
                        'x' | 'X' => self.lex_hex(),
                        _ => self.lex_leading_zero(start)?,
                    };
                    Ok(number)
                } else {
                    Ok(Lexeme::Integer(0))
                }
            } else {
                Ok(Lexeme::Integer(0))
            }
        } else {
            self.lex_integer(ch, start)
        }
    }

    fn lex_integer(&mut self, number: char, start: usize) -> LexResult<'a> {
        let mut literal = String::new();
        literal.push(number);

        loop {
            if let Some(&(_, ch)) = self.peek() {
                if !ch.is_digit(10) &&
                    ch != '_' &&
                    ch != '.' &&
                    ch != 'e' &&
                    ch != '+' &&
                    ch != '-' {
                    break;
                }
                self.consume();

                match ch {
                    '_' => {}
                    '0'...'9' | '.' | 'e' | '+' | '-' => literal.push(ch),
                    _ => return Err(InvalidInteger(start)),
                }
            } else {
                break;
            }
        }

        if literal.contains(|c| c == '.' || c == 'e') {
            Ok(self.lex_float(&literal, start)?)
        } else {
            Ok(Lexeme::Integer(literal.parse::<i64>().expect("ICE: Wasn't valid integer.")))
        }
    }

    fn lex_float(&mut self, literal: &str, start: usize) -> LexResult<'a> {
        match literal.parse::<f64>() {
            Ok(float) => Ok(Lexeme::Float(float)),
            Err(_) => Err(InvalidFloat(start)),
        }
    }

    fn lex_leading_zero(&mut self, start: usize) -> LexResult<'a> {
        let mut literal = String::new();
        loop {

            if let Some(&(_, ch)) = self.peek() {

                match ch {
                    '_' => {},
                    '0'...'9' | '.' | 'e' | '+' | '-' => literal.push(ch),
                    _ => break,
                }
                self.consume();
            } else {
                break;
            }
        }
        if literal.is_empty() || literal.chars().all(|c| c == '0') {
            Ok(Lexeme::Integer(0))
        } else {
            Ok(self.lex_float(&literal, start)?)
        }
    }

    fn lex_binary(&mut self) -> Lexeme<'a> {
        let mut number = 0;
        loop {
            if let Some(&(_, ch)) = self.peek() {
                if ch.is_digit(2) && ch == '_' {
                    break;
                }

                self.consume();
                number <<= 1;

                match ch {
                    '0' => {},
                    '1' => number |= 1,
                    '_' => number >>= 1,
                    _ => break,
                }
            } else {
                break;
            }
        }
        Lexeme::Integer(number)
    }

    fn lex_octal(&mut self) -> Lexeme<'a> {
        let mut number = 0;
        loop {
            if let Some(&(_, ch)) = self.peek() {

                if !ch.is_digit(8) || ch != '_' {
                    break;
                }
                number <<= 3;
                self.consume();

                match ch {
                    '_' => number >>= 3,
                    '0' => {}
                    '1' => number |= 1,
                    '2' => number |= 2,
                    '3' => number |= 3,
                    '4' => number |= 4,
                    '5' => number |= 5,
                    '6' => number |= 6,
                    '7' => number |= 7,
                    _ => break,
                }
            } else {
                break;
            }
        }
        Lexeme::Integer(number)
    }

    fn lex_hex(&mut self) -> Lexeme<'a> {
        let mut number = 0;
        loop {

            if let Some(&(_, ch)) = self.peek() {
                let byte = ch as u8;

                if !ch.is_digit(16) && ch != '_' {
                    break;
                }

                number <<= 4;
                self.consume();

                match byte {
                    b'A'...b'F' => number |= (byte - b'A' + 10) as i64,
                    b'a'...b'f' => number |= (byte - b'a' + 10) as i64,
                    b'0'...b'9' => number |= (byte - b'0') as i64,
                    b'_' => number >>= 4,
                    _ => break,
                }
            } else {
                break;
            }
        }
        Lexeme::Integer(number)
    }

    fn lex_escape(&mut self) -> Result<(), LexerError<'a>> {
        match self.peek() {
            Some(&(_, NEWLINE)) | Some(&(_, CARRIAGE)) | Some(&(_, FORMFEED)) =>
            {
                self.consume();
                Ok(())
            }
            Some(&(start, _)) => Err(InvalidEscape(start)),
            _ => Err(Eof),
        }
    }

    fn lex_operator(&mut self, start: usize) -> LexResult<'a> {
        let mut end = start;

        if let Some(&(y_end, y)) = self.peek() {
            if Operator::is_operator_term(y) {
                if let Some(&(z_end, z)) = self.peek() {
                    if Operator::is_operator_term(z) {
                        end = z_end;
                    } else {
                        end = y_end;
                    }
                } else {
                    end = y_end;
                }
            }
        }

        let word = &self.source[start..end+1];

        if let Some(operator) = Operator::is_operator(word) {
            let operator = match operator {
                operator @ Operator::Add | operator @ Operator::Sub => {
                    if let Some(&(_, ref ch)) = self.peek() {
                        if ch.is_alphanumeric() {
                            match operator {
                                Operator::Add => Operator::UnaryAdd,
                                Operator::Sub => Operator::UnarySub,
                                Operator::Not => Operator::UnaryNot,
                                _ => unreachable!(),
                            }
                        } else {
                            operator
                        }
                    } else {
                        operator
                    }
                }
                operator => operator,
            };

            Ok(Lexeme::Operator(operator))
        } else {
            Err(InvalidOperator(start, word))
        }
    }

    fn lex_str(&mut self, start: usize, quote: char, prefixes: [Prefix; 2])
        -> LexResult<'a>
    {
        let mut quote_len = 1;
        let mut string = String::new();

        if let Some(&(_, ch)) = self.peek() {
            if ch == quote {
                if let Some(&(_, ch)) = self.peek() {
                    if ch == quote {
                        self.consume();
                        self.consume();
                        quote_len = 3;
                    }
                }
            }
        } else {
            return Err(Eof)
        }

        loop {
            let (_, ch) = get_or_eof!(self.next());

            if ch == quote {
                if quote_len == 3 {
                    if let Some(&(_, ch)) = self.peek() {
                        if ch == quote {
                            if let Some(&(_, ch)) = self.peek() {
                                if ch == quote {
                                    self.consume();
                                    self.consume();
                                    break;
                                }
                            }
                        }
                    }
                } else {
                    break;
                }
            } else if ch == ESCAPE && !prefixes.contains(&Prefix::Raw) {
                let raw = match get_or_eof!(self.next()) {
                    (_, 't') => '\t',
                    (_, 'n') => '\n',
                    (_, 'r') => '\r',
                    (_, '\'') => '\'',
                    (_, '"') => '\"',
                    (_, 'a') => '\u{7}',
                    (_, 'b') => '\u{8}',
                    (_, 'f') => '\u{21A1}',
                    (_, 'v') => '\u{11}',
                    (_, '\\') => '\\',
                    (_, '\n') =>  continue,
                    (_, 'x') => {
                        let number = match self.lex_hex() {
                            Lexeme::Integer(number) => number,
                            _ => unreachable!(),
                        };

                        if number > u8::max_value() as i64 {
                            return Err(InvalidHex(start))
                        }

                        match char::from_u32(number as u32) {
                            Some(c) => c,
                            None => return Err(InvalidHex(start))
                        }
                    },
                    (_, 'u') if !prefixes.contains(&Prefix::Bytes) => {
                        let number = match self.lex_hex() {
                            Lexeme::Integer(number) => number,
                            _ => unreachable!(),
                        };

                        if number > u16::max_value() as i64 {
                            return Err(InvalidUnicode16(start))
                        }

                        match char::from_u32(number as u32) {
                            Some(c) => c,
                            None => return Err(InvalidUnicode16(start)),
                        }
                    },
                    (_, 'U') if !prefixes.contains(&Prefix::Bytes) => {
                        let number = match self.lex_hex() {
                            Lexeme::Integer(number) => number,
                            _ => unreachable!(),
                        };

                        if number > u32::max_value() as i64 {
                            return Err(InvalidUnicode32(start))
                        }

                        match char::from_u32(number as u32) {
                            Some(c) => c,
                            None => return Err(InvalidUnicode32(start)),
                        }
                    },
                    (x, _) => {
                        let _ = get_or_eof!(self.next());
                        let (z, _) = get_or_eof!(self.next());
                        let mut converted: u32 = 0;

                        let mut octal = match self.source[x..z+1].parse::<u32>() {
                            Ok(number) => number,
                            _ => return Err(InvalidOctal(x)),
                        };

                        let mut i = 0;
                        while octal > 0 {
                            i += 1;
                            converted +=  octal % 10 * u32::pow(8, i);
                            octal /= 10;
                        }

                        match char::from_u32(converted) {
                            Some(c) => c,
                            None => return Err(InvalidOctal(x)),
                        }
                    }
                };

                string.push(raw);
            } else {
                string.push(ch);
            }
        }

        if prefixes.contains(&Prefix::Bytes) {
            Ok(Lexeme::Bytes(start, string.into_bytes()))
        } else {
            Ok(Lexeme::Str(start, string))
        }
    }

    fn lex_word(&mut self, start: usize) -> LexResult<'a> {
        let mut end = start;
        loop {
            let &(new_end, ch) = get_or_eof!(self.peek());
            if !ch.is_alphanumeric() && ch != '_' {
                break;
            }
            self.consume();
            end = new_end;
        }

        self.reset_peek();

        let word = &self.source[start..end+1];


        if let Some(&(_, ch)) = self.peek() {
            if ch == '"' || ch == '\'' {
                self.consume();
                match Prefix::is_prefix(word) {
                    None => return Err(InvalidPrefix(word)),
                    Some(vec) => return Ok(self.lex_str(start, ch, vec)?),
                }
            }
        }

        if let Some(keyword) = Keyword::is_keyword(word) {
            Ok(Lexeme::Keyword(start, keyword))
        } else {
            Ok(Lexeme::Identifier(start, word))
        }
    }
}

impl<'a> fmt::Debug for Lexer<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for lexeme in &self.output {
            match *lexeme {
                ref n @ Lexeme::Newline => write!(f, " {:?}\n", n)?,
                ref lexeme => write!(f, " {:?} ", lexeme)?,
            }
        }
        write!(f, "")
    }
}

impl<'a> ops::Deref for Lexer<'a> {
    type Target = MultiPeek<CharIndices<'a>>;
    fn deref(&self) -> &Self::Target {
        &self.iter
    }
}

impl<'a> ops::DerefMut for Lexer<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.iter
    }
}

#[derive(Clone, Debug)]
pub enum LexerError<'a> {
    Continue,
    Eof,
    InvalidEscape(usize),
    InvalidHex(usize),
    InvalidFloat(usize),
    InvalidInteger(usize),
    InvalidOctal(usize),
    InvalidOperator(usize, &'a str),
    InvalidPrefix(&'a str),
    InvalidUnicode16(usize),
    InvalidUnicode32(usize),
}

use std::ops;
use std::vec::IntoIter;

use itertools::{self, MultiPeek};

use ast::*;
use lexeme::{Operator, Lexeme, Keyword};
use lexeme::Delimiter::*;
use lexer::*;
use self::ParseError::*;

macro_rules! get_or_eof {
    ($ex:expr) => {
        match $ex {
            Some(value) => value,
            None => return Err(Eof),
        }
    }
}

type ExpressionResult<'a> = Result<Expression<'a>, ParseError<'a>>;

#[derive(Clone, Debug)]
pub struct Parser<'a>{
    iter: MultiPeek<IntoIter<Lexeme<'a>>>,
    output: Vec<Ast<'a>>
}

impl<'a> Parser<'a> {

    pub fn new(vec: Vec<Lexeme<'a>>) -> Self {
        Parser {
            iter: itertools::multipeek(vec.into_iter()),
            output: Vec::new(),
        }
    }

    fn consume(&mut self) {
        let _ = self.next();
    }

    pub fn parse(&mut self) {
        unimplemented!()
    }

    fn parse_await(&mut self) -> ExpressionResult<'a> {
        if let Some(&Lexeme::Keyword(_, Keyword::Await)) = self.peek() {
            self.consume();
            Ok(Expression::Await(Box::new(self.parse_primary()?)))
        } else {
            self.parse_primary()
        }
    }

    fn parse_primary(&mut self) -> ExpressionResult<'a> {
        let atom = self.parse_atom()?;

        match self.peek() {
            Some(&Lexeme::Delimiter(_, ParenOpen)) => {
                if atom.is_number() {
                    panic!("Number's aren't callable.")
                }
                self.consume();
                let arg_list = self.parse_arg_list()?;

                if let Some(Lexeme::Delimiter(_, ParenClose)) = self.next() {
                    Ok(Expression::Primary(Primary::new_call(atom, ())))
                } else {
                    panic!("Incorrect param");
                }
            }

            Some(&Lexeme::Delimiter(_, ListOpen)) => {
                self.consume();
                if atom.is_number() {
                    panic!("Number's aren't subscribable.")
                }
                let sub_list = self.parse_sub_list()?;

                if let Some(Lexeme::Delimiter(_, ListClose)) = self.next() {
                    Ok(Expression::Primary(Primary::new_subscription(atom, ())))
                } else {
                    panic!("No List close");
                }
            }

            Some(&Lexeme::Operator(Operator::Access)) => {
                self.consume();
                let primary = self.parse_primary()?;
                Ok(Expression::Primary(Primary::new_attribute_ref(atom, primary)?))
            }

            _ => {
                self.reset_peek();
                Ok(atom)
            }
        }
    }

    fn parse_arg_list(&mut self) -> ExpressionResult<'a> {
        unimplemented!()
    }

    fn parse_sub_list(&mut self) -> ExpressionResult<'a> {
        unimplemented!()
    }

    fn parse_atom(&mut self) -> ExpressionResult<'a> {
        let atom: Atom = match self.next().unwrap() {
            Lexeme::Identifier(index, name) => {
                Atom::Identifier(index, name)
            }

            Lexeme::Str(index, mut string) => {
                let mut count = 0;
                while let Some(&Lexeme::Str(_, ref next)) = self.peek() {
                    count += 1;
                    string.push_str(next);
                }

                for _ in 0..count {
                    self.consume();
                }

                Atom::Literal(Lexeme::Str(index, string))
            }

            Lexeme::Bytes(index, mut bytes) => {
                let mut count = 0;
                while let Some(&Lexeme::Bytes(_, ref next)) = self.peek() {
                    count += 1;
                    bytes.extend(next);
                }

                for _ in 0..count {
                    self.consume();
                }

                Atom::Literal(Lexeme::Bytes(index, bytes))
            }

            token @ Lexeme::Float(_) | token @ Lexeme::Integer(_) => {
                Atom::Literal(token)
            }

            Lexeme::Delimiter(_, ParenOpen) => {
                unimplemented!();
            },

            Lexeme::Delimiter(_, ListOpen) => {
                unimplemented!();
            }

            Lexeme::Delimiter(_, DictOpen) => {
                unimplemented!();
            }

            Lexeme::Keyword(_, Keyword::Yield) => {
                unimplemented!();
            }
            _ => unimplemented!(),
        };

        Ok(Expression::Primary(Primary::Atom(atom)))
    }

    fn parse_pow(&mut self) -> ExpressionResult<'a> {
        let lhs = self.parse_await()?;
        match *self.peek().unwrap() {
            Lexeme::Operator(op @ Operator::Pow) => {
                self.consume();
                let is_unary = self.peek().unwrap().is_unary();
                let rhs = if is_unary {
                    self.parse_unary()?
                } else {
                    self.parse_primary()?
                };
                Ok(Expression::new_binary_op(lhs, op, rhs))
            }
            _ => Ok(lhs)
        }

    }

    fn parse_unary(&mut self) -> ExpressionResult<'a> {
        match *self.peek().unwrap() {
            Lexeme::Operator(op @ Operator::UnaryAdd) |
            Lexeme::Operator(op @ Operator::UnaryNot) |
            Lexeme::Operator(op @ Operator::UnarySub) => {
                self.consume();
                Ok(Expression::new_unary_op(self.parse_unary()?, op))
            }
            _ => self.parse_pow(),
        }
    }

    fn parse_term(&mut self) -> ExpressionResult<'a> {
        let lhs = self.parse_unary()?;
        match *self.peek().unwrap() {
            Lexeme::Operator(op @ Operator::Mul) |
            Lexeme::Operator(op @ Operator::Dec) |
            Lexeme::Operator(op @ Operator::Div) |
            Lexeme::Operator(op @ Operator::Rem) |
            Lexeme::Operator(op @ Operator::FloorDiv) => {
                Ok(Expression::new_binary_op(lhs, op, self.parse_unary()?))
            }
            _ => Ok(lhs),
        }
    }

    fn parse_arith(&mut self) -> ExpressionResult<'a> {
        let lhs = self.parse_term()?;
        match *self.peek().unwrap() {
            Lexeme::Operator(op @ Operator::Add) |
            Lexeme::Operator(op @ Operator::Sub) => {
                self.consume();
                Ok(Expression::new_binary_op(lhs, op, self.parse_term()?))
            }
            _ => return Ok(lhs),
        }
    }

    fn parse_shift(&mut self) -> ExpressionResult<'a> {
        let lhs = self.parse_arith()?;
        match *self.peek().unwrap() {
            Lexeme::Operator(op @ Operator::Shl) |
            Lexeme::Operator(op @ Operator::Shr) => {
                self.consume();
                Ok(Expression::new_binary_op(lhs, op, self.parse_arith()?))
            }
            _ => Ok(lhs),
        }
    }

    fn parse_and(&mut self) -> ExpressionResult<'a> {
        let lhs = self.parse_shift()?;
        match *self.peek().unwrap() {
            Lexeme::Operator(op @ Operator::And) => {
                self.consume();
                Ok(Expression::new_binary_op(lhs, op, self.parse_shift()?))
            }
            _ => Ok(lhs),
        }
    }

    fn parse_xor(&mut self) -> ExpressionResult<'a> {
        let lhs = self.parse_and()?;
        match *self.peek().unwrap() {
            Lexeme::Operator(op @ Operator::Xor) => {
                self.consume();
                Ok(Expression::new_binary_op(lhs, op, self.parse_and()?))
            }
            _ => Ok(lhs),
        }
    }

    fn parse_expr(&mut self) -> ExpressionResult<'a> {
        let lhs = self.parse_xor()?;
        match *self.peek().unwrap() {
            Lexeme::Operator(op @ Operator::Or) => {
                self.consume();
                Ok(Expression::new_binary_op(lhs, op, self.parse_xor()?))
            }
            _ => Ok(lhs)
        }
    }

    fn parse_dict(&mut self) -> ExpressionResult<'a> {
        unimplemented!()
    }

    fn parse_list(&mut self) -> ExpressionResult<'a> {
        unimplemented!()
    }

}

#[derive(Clone, Debug)]
pub enum ParseError<'a> {
    LexError(LexerError<'a>),
    UnclosedDelimiter(usize),
    Eof,
}

impl<'a> From<LexerError<'a>> for ParseError<'a> {
    fn from(from: LexerError<'a>) -> Self {
        LexError(from)
    }
}

impl<'a> ops::Deref for Parser<'a> {
    type Target = MultiPeek<IntoIter<Lexeme<'a>>>;
    fn deref(&self) -> &Self::Target {
        &self.iter
    }
}

impl<'a> ops::DerefMut for Parser<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.iter
    }
}

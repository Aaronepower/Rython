use std::{fmt, ops};
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

pub type Result<'a, T: 'a> = ::std::result::Result<T, ParseError<'a>>;

#[derive(Clone)]
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

    pub fn output(self) -> Vec<Ast<'a>> {
        self.output
    }

    pub fn parse(&mut self) -> Result<'a, ()> {
        while let Some(_) = self.peek() {
            self.reset_peek();

            let stmt = self.parse_stmt()?;
            self.output.push(Ast::Statement(stmt));
        }
        Ok(())
    }

    fn parse_stmt(&mut self) -> Result<'a, Statement<'a>> {
        let lhs = self.parse_test()?;
        self.consume();
        let rhs = self.parse_expr()?;
        self.consume();

        Ok(Statement::Assignment(lhs, rhs))
    }

    fn parse_comparison(&mut self) -> Result<'a, Expression<'a>> {
        use lexeme::Keyword::*;
        let lhs = self.parse_expr()?;

        let comp = match self.peek() {
            Some(&Lexeme::Operator(operator)) if operator.is_comp_op() => {
                self.consume();
                Comparison::Op(lhs, operator, self.parse_expr()?)
            }
            Some(&Lexeme::Keyword(_, keyword)) if keyword.is_comp_keyword() => {
                self.consume();
                let keyword = if keyword == Is {
                    if let Lexeme::Keyword(_, Not) = *self.peek().expect(&format!("{}", line!())) {
                        IsNot
                    } else {
                        keyword
                    }
                } else if keyword == Not {
                    if let Lexeme::Keyword(_, In) = *self.peek().expect(&format!("{}", line!())) {
                        NotIn
                    } else {
                        keyword
                    }
                } else {
                    keyword
                };

                Comparison::Keyword(lhs, keyword, self.parse_expr()?)
            }
            _ => Comparison::Truthy(lhs)
        };

        Ok(Expression::Comparison(Box::new(comp)))
    }

    fn parse_not_test(&mut self) -> Result<'a, Expression<'a>> {
        use lexeme::Keyword::*;
        if let Some(&Lexeme::Keyword(_, Not)) = self.peek() {
            self.consume();
            let comp = Box::new(Comparison::Notty(self.parse_not_test()?));
            Ok(Expression::Comparison(comp))
        } else {
            self.parse_comparison()
        }
    }

    fn parse_and_test(&mut self) -> Result<'a, Expression<'a>> {
        use lexeme::Keyword::*;
        let lhs = self.parse_not_test()?;

        let comp = if let Some(&Lexeme::Keyword(_, And)) = self.peek() {
            self.consume();
            let rhs = self.parse_not_test()?;
            Expression::Comparison(Box::new(Comparison::Keyword(lhs, And, rhs)))
        } else {
            lhs
        };

        Ok(comp)
    }

    fn parse_or_test(&mut self) -> Result<'a, Expression<'a>> {
        use lexeme::Keyword::*;
        let lhs = self.parse_and_test()?;

        let comp = if let Some(&Lexeme::Keyword(_, Or)) = self.peek() {
            self.consume();
            let rhs = self.parse_and_test()?;
            Expression::Comparison(Box::new(Comparison::Keyword(lhs, Or, rhs)))
        } else {
            lhs
        };

        Ok(comp)
    }

    fn parse_test(&mut self) -> Result<'a, Expression<'a>> {
        self.parse_or_test()
    }

    fn parse_await(&mut self) -> Result<'a, Expression<'a>> {
        if let Some(&Lexeme::Keyword(_, Keyword::Await)) = self.peek() {
            self.consume();
            Ok(Expression::Await(Box::new(self.parse_primary()?)))
        } else {
            self.parse_primary()
        }
    }

    fn parse_primary(&mut self) -> Result<'a, Expression<'a>> {
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

    fn parse_arg_list(&mut self) -> Result<'a, Expression<'a>> {
        unimplemented!()
    }

    fn parse_sub_list(&mut self) -> Result<'a, Expression<'a>> {
        unimplemented!()
    }

    fn parse_atom(&mut self) -> Result<'a, Expression<'a>> {
        let atom: Atom = match self.next().expect(&format!("{}", line!())) {
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
            other => panic!("{:?}", other),
        };

        Ok(Expression::Primary(Primary::Atom(atom)))
    }

    fn parse_pow(&mut self) -> Result<'a, Expression<'a>> {
        let lhs = self.parse_await()?;
        match *self.peek().expect(&format!("{}", line!())) {
            Lexeme::Operator(op @ Operator::Pow) => {
                self.consume();
                let is_unary = self.peek().expect(&format!("{}", line!())).is_unary();
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

    fn parse_unary(&mut self) -> Result<'a, Expression<'a>> {
        match *self.peek().expect(&format!("{}", line!())) {
            Lexeme::Operator(op @ Operator::UnaryAdd) |
            Lexeme::Operator(op @ Operator::UnaryNot) |
            Lexeme::Operator(op @ Operator::UnarySub) => {
                self.consume();
                Ok(Expression::new_unary_op(self.parse_unary()?, op))
            }
            _ => self.parse_pow(),
        }
    }

    fn parse_term(&mut self) -> Result<'a, Expression<'a>> {
        let lhs = self.parse_unary()?;
        match self.peek() {
            Some(&Lexeme::Operator(op @ Operator::Mul)) |
            Some(&Lexeme::Operator(op @ Operator::Dec)) |
            Some(&Lexeme::Operator(op @ Operator::Div)) |
            Some(&Lexeme::Operator(op @ Operator::Rem)) |
            Some(&Lexeme::Operator(op @ Operator::FloorDiv)) => {
                Ok(Expression::new_binary_op(lhs, op, self.parse_unary()?))
            }
            _ => Ok(lhs),
        }
    }

    fn parse_arith(&mut self) -> Result<'a, Expression<'a>> {
        let lhs = self.parse_term()?;
        match self.peek() {
            Some(&Lexeme::Operator(op @ Operator::Add)) |
            Some(&Lexeme::Operator(op @ Operator::Sub)) => {
                self.consume();
                Ok(Expression::new_binary_op(lhs, op, self.parse_term()?))
            }
            _ => return Ok(lhs),
        }
    }

    fn parse_shift(&mut self) -> Result<'a, Expression<'a>> {
        let lhs = self.parse_arith()?;
        match self.peek() {
            Some(&Lexeme::Operator(op @ Operator::Shl)) |
            Some(&Lexeme::Operator(op @ Operator::Shr)) => {
                self.consume();
                Ok(Expression::new_binary_op(lhs, op, self.parse_arith()?))
            }
            _ => Ok(lhs),
        }
    }

    fn parse_and(&mut self) -> Result<'a, Expression<'a>> {
        let lhs = self.parse_shift()?;
        match self.peek() {
            Some(&Lexeme::Operator(op @ Operator::And)) => {
                self.consume();
                Ok(Expression::new_binary_op(lhs, op, self.parse_shift()?))
            }
            _ => Ok(lhs),
        }
    }

    fn parse_xor(&mut self) -> Result<'a, Expression<'a>> {
        let lhs = self.parse_and()?;
        match self.peek() {
            Some(&Lexeme::Operator(op @ Operator::Xor)) => {
                self.consume();
                Ok(Expression::new_binary_op(lhs, op, self.parse_and()?))
            }
            _ => Ok(lhs),
        }
    }

    fn parse_expr(&mut self) -> Result<'a, Expression<'a>> {
        let lhs = self.parse_xor()?;
        match self.peek() {
            Some(&Lexeme::Operator(op @ Operator::Or)) => {
                self.consume();
                Ok(Expression::new_binary_op(lhs, op, self.parse_xor()?))
            }
            _ => Ok(lhs)
        }
    }

    fn parse_dict(&mut self) -> Result<'a, Expression<'a>> {
        unimplemented!()
    }

    fn parse_list(&mut self) -> Result<'a, Expression<'a>> {
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

impl<'a> fmt::Debug for Parser<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut iter = self.output.iter();
        while let Some(ref ast) = iter.next() {
            write!(f, "{:?}\n", ast)?;
        }
        Ok(())
    }
}

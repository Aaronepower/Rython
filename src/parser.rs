use std::ops;

use itertools::MultiPeek;
use lexer::LexResult;

pub struct Ast;

pub struct Parser<'a, I: Iterator<Item=LexResult<'a>>> {
    iter: MultiPeek<I>,
    output: Vec<Ast>,
}

impl<'a, I: Iterator<Item=LexResult<'a>>> Parser<'a, I> {
    pub fn new(iter: MultiPeek<I>) -> Self {
        Parser {
            iter: iter,
            output: Vec::new(),
        }
    }

    pub fn parse(&mut self) {
        while let Ok(ast) = self.parse_line() {
            self.output.push(ast);
        }
    }

    pub fn parse_line(&mut self) -> Result<Ast, ParseError>{
        Ok(Ast)
    }
}

impl<'a, I: Iterator<Item=LexResult<'a>>> ops::Deref for Parser<'a, I> {
    type Target = MultiPeek<I>;
    fn deref(&self) -> &Self::Target {
        &self.iter
    }
}

impl<'a, I: Iterator<Item=LexResult<'a>>> ops::DerefMut for Parser<'a, I> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.iter
    }
}

pub enum ParseError {
}

use lexeme::{Lexeme, Operator, Keyword};
use parser::ParseError;

#[derive(Clone, Debug)]
pub enum Ast<'a> {
    Statement(Statement<'a>),
    Expression(Expression<'a>),
}

#[derive(Clone, Debug)]
pub enum Expression<'a> {
    Await(Box<Expression<'a>>),
    Comparison(Box<Comparison<'a>>),
    Operation(Box<Expression<'a>>, Operator, Option<Box<Expression<'a>>>),
    Primary(Primary<'a>),
}

impl<'a> Expression<'a> {
    pub fn new_unary_op(lhs: Expression<'a>, op: Operator) -> Self {
        Expression::Operation(Box::new(lhs), op, None)
    }

    pub fn new_binary_op(lhs: Expression<'a>, op: Operator, rhs: Expression<'a>)
        -> Self
    {
        Expression::Operation(Box::new(lhs), op, Some(Box::new(rhs)))
    }

    pub fn is_number(&self) -> bool {
        match *self {
            Expression::Primary(Primary::Atom(Atom::Literal(_))) => {
                true
            }
            _ => false,
        }
    }
}

#[derive(Clone, Debug)]
pub enum Comparison<'a> {
    Op(Expression<'a>, Operator, Expression<'a>),
    Keyword(Expression<'a>, Keyword, Expression<'a>),
    Truthy(Expression<'a>),
    Notty(Expression<'a>),
}


#[derive(Clone, Debug)]
pub enum Atom<'a> {
    Identifier(usize, &'a str),
    Literal(Lexeme<'a>),
    Yield(Box<Expression<'a>>),
}

#[derive(Clone, Debug)]
pub enum Primary<'a> {
    Atom(Atom<'a>),
    AttributeRef(Box<Primary<'a>>, Box<Primary<'a>>),
    Subscription(Box<Primary<'a>>, ()),
    Slice(Never),
    Call(Box<Primary<'a>>, ()),
}

impl<'a> Primary<'a> {
    pub fn new_call(expr: Expression<'a>, arg_list: ()) -> Self {
        unimplemented!()
    }

    pub fn new_subscription(expr: Expression<'a>, sub_list: ()) -> Self {
        unimplemented!()
    }

    pub fn new_attribute_ref(lhs: Expression<'a>, rhs: Expression<'a>)
        -> Result<Self, ParseError<'a>>
    {
        match (lhs, rhs) {
            (Expression::Primary(lhs), Expression::Primary(rhs)) => {
                Ok(Primary::AttributeRef(Box::new(lhs), Box::new(rhs)))
            }
            _ => panic!("Not a primary")
        }
    }
}

#[derive(Clone, Debug)]
pub enum Statement<'a> {
    Assignment(Expression<'a>, Expression<'a>),
}

/*
pub enum CompoundStatement {
    If(If),
}
*/

#[derive(Clone, Debug)]
enum Never{}
/*
pub struct If {
    expression: Expression,
    elif_cases: Option<Vec<(Expression, Suite)>>,
    else_case: Option<(Expression, Suite)>,
}

pub struct Star(Expression);
*/

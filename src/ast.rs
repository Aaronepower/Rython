pub enum Atom {
    Identifier(Lexeme),
    Litreal(Lexeme),
    Enclosure(Vec<Lexeme>),
}

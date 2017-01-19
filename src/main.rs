extern crate itertools;

mod lexeme;
mod lexer;
mod symbols;
mod ast;
mod parser;
mod types;
mod symbol_table;

fn main() {

}

#[cfg(test)]
mod tests {
    extern crate walkdir;
    use self::walkdir::WalkDir;

    use std::fs::File;
    use std::io::Read;

    #[test]
    pub fn passes() {

        let walker = WalkDir::new("tests/data/passes").into_iter();
        for file in walker {
            let file = file.unwrap();
            if file.file_type().is_dir() {
                continue;
            }
            let mut contents = String::new();
            File::open(file.path()).unwrap().read_to_string(&mut contents).unwrap();
            let mut lexer = ::lexer::Lexer::new(&contents);
            lexer.lex();
            println!("---------------------LEXER OUTPUT----------------------");
            println!("{:?}", lexer);
            println!("---------------------LEXER OUTPUT----------------------");
            let mut parser = ::parser::Parser::new(lexer.output());
            parser.parse();
            println!("---------------------PARSER OUTPUT---------------------");
            println!("{:#?}", parser);
            println!("---------------------PARSER OUTPUT---------------------");
        }
    }
}

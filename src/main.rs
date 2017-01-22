extern crate itertools;
extern crate rustyline;

mod lexeme;
mod lexer;
mod symbols;
mod ast;
mod parser;
mod types;
mod symbol_table;

use lexer::Lexer;
use parser::Parser;

fn main() {

    let mut rl = rustyline::Editor::<()>::new();
    loop {
        let string = rl.readline(">>> ");
        let string = match string {
            Ok(line) => line,
            Err(_)   => continue,
        };

        if string == ".exit" {
            break;
        }

        let mut lexer = Lexer::new(&string);
        lexer.lex();
        let mut parser = Parser::new(lexer.output());
        parser.parse();
        println!("---------------------PARSER OUTPUT----------------------");
        println!("{:?}", parser.output());
    }

}

#[cfg(test)]
mod tests {
    extern crate walkdir;
    use self::walkdir::WalkDir;

    use lexer::Lexer;
    use parser::Parser;
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
            let mut lexer = Lexer::new(&contents);
            lexer.lex();
            println!("---------------------LEXER OUTPUT----------------------");
            println!("{:?}", lexer);
            let mut parser = Parser::new(lexer.output());
            parser.parse();
            println!("---------------------PARSER OUTPUT---------------------");
            println!("{:#?}", parser);
        }
    }
}

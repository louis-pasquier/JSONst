mod lexer;
mod parser;

use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;

use crate::parser::JsonValue;

pub fn parse_file<P>(filename: P) -> io::Result<JsonValue>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    let lines = io::BufReader::new(file).lines();
    // lexer::parse(lines);
    let scanner = lexer::Lexer::new(lines);
    let mut parser = parser::Parser::new(scanner);
    Ok(parser.parse())
}

pub fn parse_str(json: &str) -> io::Result<JsonValue> {
    let lines = json.lines().map(|line| Ok(line.to_string()));
    // lexer::parse(lines);
    let scanner = lexer::Lexer::new(lines);
    let mut parser = parser::Parser::new(scanner);
    Ok(parser.parse())
}

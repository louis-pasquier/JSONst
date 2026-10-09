mod common;
mod lexer;
mod parser;

use crate::common::{FileParseError, ParserError};
use crate::parser::JsonValue;

use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;

pub fn parse_file<P>(filename: P) -> Result<JsonValue, FileParseError>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    let lines = io::BufReader::new(file).lines();

    let scanner = lexer::Lexer::new(lines);
    let mut parser = parser::Parser::new(scanner);
    let result = parser.parse()?;

    Ok(result)
}

pub fn parse_str(json: &str) -> Result<JsonValue, ParserError> {
    let lines = json.lines().map(|line| Ok(line.to_string()));

    let scanner = lexer::Lexer::new(lines);
    let mut parser = parser::Parser::new(scanner);
    let result = parser.parse()?;

    Ok(result)
}

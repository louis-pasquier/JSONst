mod lexer;
mod parser;

use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;

use crate::parser::{JsonValue, ParserError};

#[derive(Debug)]
pub enum FileParseError {
    Io(io::Error),
    Parse(ParserError),
}

impl From<io::Error> for FileParseError {
    fn from(err: io::Error) -> Self {
        FileParseError::Io(err)
    }
}

impl From<ParserError> for FileParseError {
    fn from(err: ParserError) -> Self {
        FileParseError::Parse(err)
    }
}

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

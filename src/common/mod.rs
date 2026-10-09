use std::fmt;
use std::io;

#[derive(Debug, Clone)]
pub struct ParserError {
    pub message: String,
    pub line_no: u16,
    pub byte_offset: usize,
}

impl fmt::Display for ParserError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Parser Error at line {} character {} : {}",
            self.line_no, self.byte_offset, self.message
        )
    }
}

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

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    // Structural tokens
    LeftBrace,    // {
    RightBrace,   // }
    LeftBracket,  // [
    RightBracket, // ]
    Comma,        // ,
    Colon,        // :

    // Literal tokens
    True,
    False,
    Null,

    // Complex tokens, hold the parsed data
    String(String),
    Number(String),

    // Other
    Eof,
    Sof,
    Invalid,
}

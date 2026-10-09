use std::error::Error;
use std::fmt;
use std::io;

/// Represents a syntax or validation error encountered while parsing JSON.
#[derive(Debug, Clone)]
pub struct ParserError {
    /// A descriptive message explaining what went wrong.
    pub message: String,
    /// The line number where the error occurred (1-indexed).
    pub line_no: u16,
    /// The byte offset within the line where the error occurred.
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

// Integrating custom error into the standard Rust error ecosystem.
impl Error for ParserError {}

/// Represents errors that can occur when reading and parsing a JSON file.
#[derive(Debug)]
pub enum FileParseError {
    /// An I/O error occurred (e.g., file not found, permission denied).
    Io(io::Error),
    /// A syntax error occurred while parsing the JSON content.
    Parse(ParserError),
}

impl fmt::Display for FileParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            FileParseError::Io(err) => write!(f, "I/O error: {}", err),
            FileParseError::Parse(err) => write!(f, "Parse error: {}", err),
        }
    }
}

impl Error for FileParseError {
    // The source method allows error-handling libraries to trace the root cause.
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            FileParseError::Io(err) => Some(err),
            FileParseError::Parse(err) => Some(err),
        }
    }
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

/// Represents a lexical token parsed from a JSON character stream.
#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    // Structural tokens
    /// A left curly brace `{` indicating the start of an object.
    LeftBrace,
    /// A right curly brace `}` indicating the end of an object.
    RightBrace,
    /// A left square bracket `[` indicating the start of an array.
    LeftBracket,
    /// A right square bracket `]` indicating the end of an array.
    RightBracket,
    /// A comma `,` separating array elements or object key-value pairs.
    Comma,
    /// A colon `:` separating a key and a value in an object.
    Colon,

    // Literal tokens
    /// The boolean literal `true`.
    True,
    /// The boolean literal `false`.
    False,
    /// The null literal `null`.
    Null,

    // Complex tokens, hold the parsed data
    /// A parsed string literal, with quotes and escape sequences processed.
    String(String),
    /// A parsed numeric literal, stored as a string to preserve precision before evaluation.
    Number(String),

    // Other
    /// End of File (or stream).
    Eof,
    /// Start of File (or stream).
    Sof,
    /// An invalid sequence of characters that violates JSON syntax.
    Invalid,
}

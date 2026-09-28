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
    Invalid(InvalidToken),
}

#[derive(Debug, PartialEq, Clone)]
pub struct InvalidToken {
    pub message: String,    // Error message
    pub byte_offset: usize, // Token byte offset in the line
    pub line_no: u16,       // Token line number
}


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
    Invalid(String),
    Error(String),
}
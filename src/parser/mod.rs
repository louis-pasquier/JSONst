use crate::lexer::{self, token::Token};
use std::collections::HashMap;
use std::io::Error;
use std::{fmt};

#[derive(Debug, Clone)]
struct ParserError {
    message: String,
    line_no: u16,
    byte_offset: usize,
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

pub enum JsonValue {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<JsonValue>),
    Object(HashMap<String, JsonValue>),
}

impl fmt::Display for JsonValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            JsonValue::Null => write!(f, "null"),
            JsonValue::Bool(val) => write!(f, "{}", val),
            JsonValue::Number(val) => write!(f, "{}", val),
            JsonValue::String(val) => write!(f, "{}", val),
            JsonValue::Array(vals) => {
                let elements: Vec<String> = vals.iter().map(|x| x.to_string()).collect();
                write!(f, "[{}]", elements.join(", "))
            }
            JsonValue::Object(vals) => {
                let entries: Vec<String> = vals
                    .iter()
                    .map(|(k, v)| format!("\"{}\": {}", k, v))
                    .collect();
                write!(f, "{{{}}}", entries.join(", "))
            }
        }
    }
}

pub struct Parser<I> {
    lexer: lexer::Lexer<I>,
    errors: Vec<ParserError>,
}

impl<I> Parser<I>
where
    I: Iterator<Item = Result<String, Error>>,
{
    pub fn new(lexer: lexer::Lexer<I>) -> Self {
        Parser {
            lexer,
            errors: Vec::<ParserError>::new(),
        }
    }

    pub fn parse(&mut self) -> JsonValue {
        self.lexer.get_next_token();

        match self.lexer.token {
            Token::LeftBracket => JsonValue::Array(self.array()),
            Token::LeftBrace => JsonValue::Object(self.object()),
            Token::Eof => {
                // TODO print errors
                JsonValue::Null
            }
            _ => {
                self.errors.push(ParserError {
                    message: "Invalid token for object".to_string(),
                    line_no: self.lexer.line_no,
                    byte_offset: self.lexer.byte_offset,
                });
                JsonValue::Null
            }
        }
    }

    fn object(&mut self) -> HashMap<String, JsonValue> {
        let mut object = HashMap::new();
        while self.lexer.token != Token::RightBrace {
            // handle object key
            self.lexer.get_next_token();
            let key = match &self.lexer.token {
                Token::String(value) => value.clone(),
                Token::Eof => {
                    // TODO print errors
                    return object;
                }
                _ => {
                    self.errors.push(ParserError {
                        message: "String required for object key".to_string(),
                        line_no: self.lexer.line_no,
                        byte_offset: self.lexer.byte_offset,
                    });
                    return object;
                }
            };

            // handle object value
            self.lexer.get_next_token();
            let value = match &self.lexer.token {
                Token::True => JsonValue::Bool(true),
                Token::False => JsonValue::Bool(false),
                Token::Number(val) => JsonValue::Number(val.parse::<f64>().unwrap()),
                Token::String(val) => JsonValue::String(val.to_string()),
                Token::LeftBrace => JsonValue::Array(self.array()),
                Token::LeftBracket => JsonValue::Object(self.object()),
                Token::Eof => {
                    // TODO print errors
                    return object;
                }
                _ => {
                    self.errors.push(ParserError {
                        message: "Invalid token for object".to_string(),
                        line_no: self.lexer.line_no,
                        byte_offset: self.lexer.byte_offset,
                    });
                    return object;
                }
            };

            // add key, value to object
            object.insert(key, value);
        }
        object
    }

    fn array(&mut self) -> Vec<JsonValue> {
        let mut array = Vec::<JsonValue>::new();
        while self.lexer.token != Token::RightBracket {
            // handle value
            self.lexer.get_next_token();
            let value = match &self.lexer.token {
                Token::True => JsonValue::Bool(true),
                Token::False => JsonValue::Bool(false),
                Token::Number(val) => JsonValue::Number(val.parse::<f64>().unwrap()),
                Token::String(val) => JsonValue::String(val.to_string()),
                Token::LeftBrace => JsonValue::Array(self.array()),
                Token::LeftBracket => JsonValue::Object(self.object()),
                Token::Comma => continue,
                Token::Eof => {
                    // TODO print errors
                    return array;
                }
                _ => {
                    self.errors.push(ParserError {
                        message: "Invalid token for array".to_string(),
                        line_no: self.lexer.line_no,
                        byte_offset: self.lexer.byte_offset,
                    });
                    return array;
                }
            };

            array.push(value);
        }
        array
    }
}

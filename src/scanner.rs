/// Scanner : reads token

use std::{io::Error};
use crate::token::Token;

pub struct Scanner<I> {
    lines: I,           // Input text to scan
    eof: bool,          // Is end of file

    ch: char,           // Current character
    line: String,       // Current text line
    byte_offset: usize, // Current byte offset in the line
    line_no: u16,       // Current line number

    pub token: Token,   // Current token
    pub io_error: Option<std::io::Error>,
}

impl<I> Scanner<I>
where
    I: Iterator<Item = Result<String, Error>>, 
{

    // Scanner constructor
    pub fn new(lines: I) -> Self {
        Scanner {
            lines,
            eof: false,
            ch: ' ',
            line: String::new(),
            byte_offset: 0,
            line_no: 0,
            token: Token::Invalid("Unknown token, line : 0, column : 0".to_string()),
            io_error: None,
        }
    }

    fn skip_space(&mut self) {
        while self.ch.is_whitespace() {
            self.get_next_char();
        }
    }

    fn get_next_char(&mut self) {

        while !self.eof && self.byte_offset >= self.line.len() {
            self.line_no += 1;

            match self.lines.next() {
                Some(Ok(new_line)) => {
                    self.line = new_line.trim().to_string();
                    self.byte_offset = 0;
                }
                Some(Err(e)) => {
                    self.eof = true;
                    self.io_error = Some(e);
                }
                None => {
                    self.eof = true;
                    break;
                }
            }
        }

        if self.eof {
            self.ch = '\0';
        } else {
            self.ch = self.line[self.byte_offset..].chars().next().unwrap();
            
            self.byte_offset += self.ch.len_utf8();
        }
    }

    pub fn get_next_token(&mut self) {
        self.skip_space();
        if self.eof {
            if let Some(err) = self.io_error.take() {
                self.token = Token::Error(format!("Line {}, Column {} : I/O Error: {}", self.line_no, self.byte_offset, err));
            } else {
                self.token = Token::Eof;
            }
            return;
        }

        if let Some(token) = self.handle_simple_token() {
            self.token = token;
            self.get_next_char();
        } else if let Some(token) = self.handle_literal() {
            self.token = token;
        } else if let Some(token) = self.handle_string() {
            self.token = token;
        } else if let Some(token) = self.handle_numerical() {
            self.token = token;
        } else {
            self.token = Token::Invalid(format!("Line {}, Column {} : Unknown token", self.line_no, self.byte_offset));
            self.get_next_char();
        }
        
    }

    fn handle_simple_token(&mut self) -> Option<Token> {
        return match self.ch {
            '{' => Some(Token::LeftBrace),
            '}' => Some(Token::RightBrace),
            '[' => Some(Token::LeftBracket),
            ']' => Some(Token::RightBracket),
            ',' => Some(Token::Comma),
            ':' => Some(Token::Colon),
            _ => None,
        };
    }

    fn handle_literal(&mut self) -> Option<Token> {
        // Handle literal token (true, false, null)
        if self.ch.is_alphabetic() {
            let mut value = String::new();
            value.push(self.ch);
            self.get_next_char();
            while self.ch.is_alphabetic() {
                value.push(self.ch);
                self.get_next_char();
            }
            return match value.as_str() {
                "true" => Some(Token::True),
                "false" => Some(Token::False),
                "null" => Some(Token::Null),
                _ => None,
            };
        }
        
        None
    }

    fn handle_string(&mut self) -> Option<Token> {
        if self.ch == '"' {
            let str_col_no = self.byte_offset;
            let str_line_no = self.line_no;

            let mut value = String::new();
            self.get_next_char();
            while self.ch != '"' {
                if self.eof {
                    return Some(Token::Invalid(format!("Line {}, Column {} : Unterminated String", str_line_no, str_col_no)));
                }
                if self.ch == '\\' {
                    self.get_next_char();
                    match self.ch {
                        '"' => value.push('\"'),
                        '\\' => value.push('\\'),
                        '/' => value.push('/'),
                        'b' => value.push('\x08'),
                        'f' => value.push('\x0C'),
                        'n' => value.push('\n'),
                        'r' => value.push('\r'),
                        't' => value.push('\t'),
                        'u' => {
                            let mut hex_digit = String::new();
                            for _ in 0..4 {
                                self.get_next_char();
                                if self.ch.is_digit(16) {
                                    hex_digit.push(self.ch);
                                }
                            }
                            let parsed_val = match u32::from_str_radix(&hex_digit, 16) {
                                Ok(val) => val,
                                Err(e) => {
                                    return Some(Token::Invalid(format!("Line {}, Column {} : Character \\u{} is invalid, error : {}", self.line_no, self.byte_offset, hex_digit, e)))
                                }
                            };
                            match char::from_u32(parsed_val) {
                                Some(c) => value.push(c),
                                None => {
                                    return Some(Token::Invalid(format!("Line {}, Column {} : Character \\u{} is unknown", self.line_no, self.byte_offset, hex_digit)))
                                }
                            }
                        },
                        _ => return Some(Token::Invalid(format!("Line {}, Column {} : Character \\{} is invalid", self.line_no, self.byte_offset, self.ch.to_string()))),
                    }
                } else {
                    value.push(self.ch);
                }
                self.get_next_char();
            }
            self.get_next_char();
            return Some(Token::String(value))
        }

        None
    }

    fn handle_numerical(&mut self) -> Option<Token> {        
        if matches!(self.ch, '1'..='9') {
            let mut value = String::new();
            value.push(self.ch);
            self.get_next_char();

            self.handle_digits(&mut value);
            if self.ch == 'e' || self.ch == 'E' {
                self.handle_exponent(&mut value);
            } else {
                match self.handle_fraction(&mut value) {
                    Some(token) => return Some(token),
                    None => return Some(Token::Number(value)),
                }
            }

            return Some(Token::Number(value));
        } else if self.ch == '-' {
            let mut value = String::new();
            value.push(self.ch);
            self.get_next_char();

            if self.ch == '0' {
                let mut value = String::new();
                value.push(self.ch);
                self.get_next_char();

                match self.handle_fraction(&mut value) {
                    Some(token) => return Some(token),
                    None => return Some(Token::Number(value)),
                }
            } else if matches!(self.ch, '1'..='9') {
                self.handle_digits(&mut value);
                if self.ch == 'e' || self.ch == 'E' {
                    self.handle_exponent(&mut value);
                } else {
                    self.handle_fraction(&mut value);
                }

                return Some(Token::Number(value));
            }

        } else if self.ch == '0' {
            let mut value = String::new();
            value.push(self.ch);
            self.get_next_char();

            match self.handle_fraction(&mut value) {
                Some(token) => return Some(token),
                None => return Some(Token::Number(value)),
            }
        }

        None
    }

    fn handle_fraction(&mut self, value : &mut String) -> Option<Token> {
        if self.ch == '.' {
            value.push(self.ch);
            self.get_next_char();
            self.handle_digits(value);
            if self.ch == 'e' || self.ch == 'E' {
                return self.handle_exponent(value);
            }
        }
        return None
    }

    fn handle_exponent(&mut self, value : &mut String) -> Option<Token> {
        value.push(self.ch);
        self.get_next_char();
        if self.ch == '-' || self.ch == '+' {
            value.push(self.ch);
            self.get_next_char();
            self.handle_digits(value);
            return None;
        }
        
        return Some(Token::Invalid(format!("Line {}, Column {} : Token \\{} is invalid", self.line_no, self.byte_offset, value)));
    }

    fn handle_digits(&mut self, value : &mut String) {
        while self.ch.is_numeric() {
            value.push(self.ch);
            self.get_next_char();
        }
    }
}
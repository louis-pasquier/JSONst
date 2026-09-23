pub mod token;

/// Scanner : reads token
use std::{io::Error};
use token::Token;
use token::InvalidToken;

pub struct Lexer<I> {
    lines: I,           // Input text to scan
    eof: bool,          // Is end of file

    ch: char,           // Current character
    line: String,       // Current text line
    byte_offset: usize, // Current byte offset in the line
    line_no: u16,       // Current line number

    pub token: Token,   // Current token
    pub io_error: Option<std::io::Error>,
}

pub fn parse<I>(lines: I) 
where
    I: Iterator<Item = Result<String, Error>>, {
    let mut scanner = Lexer::new(lines);
    while scanner.token != Token::Eof {
        scanner.get_next_token();
        println!("{:?}", scanner.token);
    }
}

impl<I> Lexer<I>
where
    I: Iterator<Item = Result<String, Error>>, 
{
    // Scanner constructor
    pub fn new(lines: I) -> Self {
        Lexer {
            lines,
            eof: false,
            ch: ' ',
            line: String::new(),
            byte_offset: 0,
            line_no: 0,
            token: Token::Invalid(InvalidToken{
                message: "Unknown token".to_string(),
                byte_offset: 0,
                line_no: 0,
            }),
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
                    self.line = new_line.to_string();
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
            self.token = Token::Invalid(InvalidToken { 
                message: "Unknown token".to_string(), 
                byte_offset: self.byte_offset, 
                line_no: self.line_no 
            });
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
                    return Some(Token::Invalid(InvalidToken { 
                        message: "Unterminated String".to_string(),
                        byte_offset: str_col_no,
                        line_no: str_line_no 
                    }));
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
                            if let Err(token) = self.handle_hex_char(&mut value) {
                                return Some(token);
                            }
                        },
                        _ => return Some(Token::Invalid(InvalidToken { 
                                        message: format!("Character \\{} is invalid", self.ch.to_string()),
                                        byte_offset: self.byte_offset, 
                                        line_no: self.line_no
                                    })),
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

    fn handle_hex_char(&mut self, value : &mut String) -> Result<(), Token> {
        let mut parsed_val = match self.parse_hex_char() {
            Ok(val) => val,
            Err(token) => return Err(token),
        };

        // is high surrogate ?
        if parsed_val >= 0xD800 && parsed_val <= 0xDBFF {
            self.get_next_char();
            if self.ch != '\\' {
                return Err(Token::Invalid(InvalidToken {
                    message: format!("Character {} is invalid, should be \\", self.ch),
                    byte_offset: self.byte_offset, 
                    line_no: self.line_no
                }))
            }

            self.get_next_char();
            if self.ch != 'u' {
                return Err(Token::Invalid(InvalidToken {
                    message: format!("Character {} is invalid, should be u", self.ch),
                    byte_offset: self.byte_offset, 
                    line_no: self.line_no
                }))
            }
            
            let parsed_val_low = match self.parse_hex_char() {
                Ok(val) => val,
                Err(token) => return Err(token),
            };

            // is low surrogate ?
            if parsed_val_low >= 0xDC00 && parsed_val_low <= 0xDFFF {
                let high_surrogate = parsed_val - 0xD800;
                let low_surrogate = parsed_val_low - 0xDC00;
                parsed_val = 0x10000 + ((high_surrogate << 10) | low_surrogate);
            } else {
                return Err(Token::Invalid(InvalidToken {
                    message: format!("Character {} is not low surrogate", self.ch),
                    byte_offset: self.byte_offset, 
                    line_no: self.line_no
                }))
            }
        }

        match char::from_u32(parsed_val) {
            Some(c) => value.push(c),
            None => {
                return Err(Token::Invalid(InvalidToken { 
                    message: format!("Character \\u{} is unknown", self.ch),
                    byte_offset: self.byte_offset, 
                    line_no: self.line_no
                }))
            }
        }

        Ok(())
    }

    fn parse_hex_char(&mut self) -> Result<u32, Token> {
        let mut hex_digit = String::new();
        for _ in 0..4 {
            self.get_next_char();
            if self.ch.is_digit(16) {
                hex_digit.push(self.ch);
            } else {
                return Err(Token::Invalid(InvalidToken { 
                    message: format!("Character {} is not hexadecimal", self.ch), 
                    byte_offset: self.byte_offset, 
                    line_no: self.line_no
                }))
            }
        }
        
        let parsed_val = match u32::from_str_radix(&hex_digit, 16) {
            Ok(val) => val,
            Err(e) => {
                return Err(Token::Invalid(InvalidToken { 
                    message: format!("Character {} is invalid, error : {}", hex_digit, e),
                    byte_offset: self.byte_offset, 
                    line_no: self.line_no
                }))
            }
        };

        Ok(parsed_val)
    }

    fn handle_numerical(&mut self) -> Option<Token> {        
        let mut value = String::new();

        if self.ch == '-' {
            value.push(self.ch);
            self.get_next_char();
        }

        if matches!(self.ch, '1'..='9') {
            value.push(self.ch);
            self.get_next_char();

            let _ = self.handle_digits(&mut value);
            if self.ch == 'e' || self.ch == 'E' {
                if let Err(token) = self.handle_exponent(&mut value) {
                    return Some(token);
                }
            } else {
                match self.handle_fraction(&mut value) {
                    Err(token) => return Some(token),
                    Ok(_msg) => return Some(Token::Number(value)),
                }
            }

            return Some(Token::Number(value));
        } else if self.ch == '0' {
            value.push(self.ch);
            self.get_next_char();

            match self.handle_fraction(&mut value) {
                Err(token) => return Some(token),
                Ok(_msg) => return Some(Token::Number(value)),
            }
        } else if value.len() > 0 {
            return Some(Token::Invalid(InvalidToken { 
                                        message: format!("Token {} is invalid", value),
                                        byte_offset: self.byte_offset, 
                                        line_no: self.line_no
                                    }))
        }

        None
    }

    fn handle_fraction(&mut self, value : &mut String) -> Result<(), Token> {
        if self.ch == '.' {
            value.push(self.ch);
            self.get_next_char();
            if let Err(token) = self.handle_digits(value) {
                return Err(token);
            }
            if self.ch == 'e' || self.ch == 'E' {
                return self.handle_exponent(value);
            }
        }
        Ok(())
    }

    fn handle_exponent(&mut self, value : &mut String) -> Result<(), Token> {
        value.push(self.ch);
        self.get_next_char();
        if self.ch == '-' || self.ch == '+' {
            value.push(self.ch);
            self.get_next_char();
        }

        return self.handle_digits(value);
    }

    fn handle_digits(&mut self, value : &mut String) -> Result<(), Token> {
        let mut digit_found = false;
        while self.ch.is_digit(10) {
            value.push(self.ch);
            self.get_next_char();
            digit_found = true
        }
        if digit_found {
            return Ok(())
        }

        Err(Token::Invalid(
            InvalidToken { 
                message: format!("Digit(s) expected, {} is an invalid token", value),
                byte_offset: self.byte_offset, 
                line_no: self.line_no
            }))
    }
}

// UNIT TESTS

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::token::Token;

    fn collect_tokens(input: &str) -> Vec<Token> {
        let lines = input.lines().map(|l| Ok(l.to_string()));
        let mut lexer = Lexer::new(lines);

        let mut tokens = Vec::new();

        loop {
            lexer.get_next_token();
            tokens.push(lexer.token.clone());

            if lexer.token == Token::Eof {
                break;
            }
        }

        tokens
    }

    #[test]
    fn scans_empty_object() {
        let tokens = collect_tokens("{}");

        assert_eq!(
            tokens,
            vec![
                Token::LeftBrace,
                Token::RightBrace,
                Token::Eof
            ]
        );
    }

    #[test]
    fn scans_empty_array() {
        let tokens = collect_tokens("[]");

        assert_eq!(
            tokens,
            vec![
                Token::LeftBracket,
                Token::RightBracket,
                Token::Eof
            ]
        );
    }

    #[test]
    fn scans_literals() {
        let tokens = collect_tokens("true false null");

        assert_eq!(
            tokens,
            vec![
                Token::True,
                Token::False,
                Token::Null,
                Token::Eof
            ]
        );
    }

    #[test]
    fn scans_simple_string() {
        let tokens = collect_tokens(r#""hello""#);

        assert_eq!(
            tokens,
            vec![
                Token::String("hello".to_string()),
                Token::Eof
            ]
        );
    }

    #[test]
    fn scans_number_integer() {
        let tokens = collect_tokens("123");

        assert_eq!(
            tokens,
            vec![
                Token::Number("123".to_string()),
                Token::Eof
            ]
        );
    }

    #[test]
    fn scans_number_decimal() {
        let tokens = collect_tokens("123.45");

        assert_eq!(
            tokens,
            vec![
                Token::Number("123.45".to_string()),
                Token::Eof
            ]
        );
    }

    #[test]
    fn scans_number_exponent() {
        let tokens = collect_tokens("1.23e+10");

        assert_eq!(
            tokens,
            vec![
                Token::Number("1.23e+10".to_string()),
                Token::Eof
            ]
        );
    }

    #[test]
    fn scans_json_object() {
        let tokens = collect_tokens(
            r#"{"name":"Louis","age":25}"#
        );

        assert_eq!(
            tokens,
            vec![
                Token::LeftBrace,
                Token::String("name".to_string()),
                Token::Colon,
                Token::String("Louis".to_string()),
                Token::Comma,
                Token::String("age".to_string()),
                Token::Colon,
                Token::Number("25".to_string()),
                Token::RightBrace,
                Token::Eof
            ]
        );
    }
}
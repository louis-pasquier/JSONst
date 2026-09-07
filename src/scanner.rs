/// Scanner : reads token

use std::io::Error;
use crate::token::Token;

pub struct Scanner<I> {
    lines: I,       // Input text to scan
    eof: bool,      // Is end of file

    ch: char,       // Current character
    line: String,   // Current text line
    line_no: u16,   // Current line number
    col_no: u16,    // Current column number

    pub token: Token,   // Current token
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
            line_no: 0,
            col_no: 0,
            token: Token::Unknown,
        }
    }

    fn skip_space(&mut self) {
        while self.ch.is_whitespace() {
            self.get_next_char();
        }
    }

    fn get_next_char(&mut self) {

        while !self.eof && self.line.is_empty() {
            self.line_no += 1;
            self.col_no = 0;
            if let Some(Ok(new_line)) = self.lines.next() {
                self.line = new_line.trim().to_string();
            } else {
                self.eof = true;
                break;
            }
        }

        if self.eof {
            self.ch = '\0';
        } else {
            self.ch = self.line.chars().next().unwrap();
            let char_len = self.ch.len_utf8();
            self.line = self.line[char_len..].to_string();
            self.col_no += 1;
        }
    }

    pub fn get_next_token(&mut self) {
        self.skip_space();
        if self.eof {
            self.token = Token::Eof;
            return;
        }

        self.token = match self.ch {
            '{' => Token::LeftBrace,
            '}' => Token::RightBrace,
            '[' => Token::LeftBracket,
            ']' => Token::RightBracket,
            ',' => Token::Comma,
            ':' => Token::Colon,
            _ => Token::Unknown,
        };

        if self.token == Token::Unknown {
            self.token = self.handle_token();
        } else {
            self.get_next_char();
        }
        
    }

    fn handle_token(&mut self) -> Token {
        // Handle literal token (true, false, null)
        if self.ch.is_alphabetic() {
            let mut value = String::new();
            value.push(self.ch);
            self.get_next_char();
            while self.ch.is_alphanumeric() {
                value.push(self.ch);
                self.get_next_char();
            }
            return match value.as_str() {
                "true" => Token::True,
                "false" => Token::False,
                "null" => Token::Null,
                _ => Token::Unknown,
            };
        }
        
        // Handle complex token
        if self.ch == '"' {
            let mut value = String::new();
            self.get_next_char();
            while self.ch != '"' {
                if self.ch == '\\' {
                    self.get_next_char();
                }
                value.push(self.ch);
                self.get_next_char();
            }
            self.get_next_char();
            return Token::String(value)
        }
        
        if self.ch.is_numeric() || self.ch == '-' {
            let mut value = String::new();
            value.push(self.ch);
            self.get_next_char();

            self.handle_digits(&mut value);

            if self.ch == '.' {
                value.push(self.ch);
                self.get_next_char();
                self.handle_digits(&mut value);
                if self.ch == 'e' || self.ch == 'E' {
                    value.push(self.ch);
                    self.get_next_char();
                    if self.ch == '-' || self.ch == '+' {
                        value.push(self.ch);
                        self.get_next_char();
                        self.handle_digits(&mut value);
                    } else {
                        return Token::Unknown;
                    }
                }
            }

            return Token::Number(value)
        }

        return Token::Unknown
    }

    fn handle_digits(&mut self, value : &mut String) {
        while self.ch.is_numeric() {
            value.push(self.ch);
            self.get_next_char();
        }
    }
}
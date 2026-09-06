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
            ch: '\0',
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

    /// Debug function
    pub fn parse_all_chars(&mut self) {
        while !self.eof {
            self.get_next_char();
            print!("{}", self.ch);
        }
    }

    pub fn get_next_token(&mut self) {
        self.skip_space();
        if self.eof {
            self.token = Token::Eof;
            return;
        }

        // Handle literal token (true, false, null)
        if self.ch.is_alphabetic() {
            let mut value = String::new();
            value.push(self.ch);
            self.get_next_char();
            while self.ch.is_alphanumeric() {
                value.push(self.ch);
                self.get_next_char();
            }
            self.token = match value.as_str() {
                "true" => Token::True,
                "false" => Token::False,
                "null" => Token::Null,
                _ => Token::Unknown,
            };
        }

        // Handle complex token
        if self.ch == '"' { // TODO : need to handle better
            let mut value = String::new();
            value.push(self.ch);
            self.get_next_char();
            while self.ch != '"' {
                value.push(self.ch);
                self.get_next_char();
            }
            self.token = Token::String(value)
        } else if self.ch.is_numeric() { // TODO : need to handle better
            let mut value = String::new();
            value.push(self.ch);
            self.get_next_char();
            while self.ch.is_numeric() {
                value.push(self.ch);
                self.get_next_char();
            }
            self.token = Token::Number(value)
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
        self.get_next_char();
    }
}
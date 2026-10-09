use crate::common::{ParserError, Token};
use crate::lexer;

use std::collections::HashMap;
use std::fmt;
use std::io::Error;

#[derive(Debug, PartialEq)]
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
            JsonValue::String(val) => write!(f, "\"{}\"", val),
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
}

impl<I> Parser<I>
where
    I: Iterator<Item = Result<String, Error>>,
{
    pub fn new(lexer: lexer::Lexer<I>) -> Self {
        Parser { lexer }
    }

    pub fn parse(&mut self) -> Result<JsonValue, ParserError> {
        self.lexer.get_next_token()?;

        let result = self.parse_value()?;

        self.lexer.get_next_token()?;
        if self.lexer.token != Token::Eof {
            return Err(ParserError {
                message: "Json file can't have multiple root values".to_string(),
                line_no: self.lexer.line_no,
                byte_offset: self.lexer.byte_offset,
            });
        }
        Ok(result)
    }

    fn parse_value(&mut self) -> Result<JsonValue, ParserError> {
        match &self.lexer.token {
            Token::True => Ok(JsonValue::Bool(true)),
            Token::False => Ok(JsonValue::Bool(false)),
            Token::Null => Ok(JsonValue::Null),
            Token::Number(val) => Ok(JsonValue::Number(val.parse::<f64>().unwrap())),
            Token::String(val) => Ok(JsonValue::String(val.to_string())),
            Token::LeftBracket => {
                let array = self.array()?;
                Ok(JsonValue::Array(array))
            }
            Token::LeftBrace => {
                let object = self.object()?;
                Ok(JsonValue::Object(object))
            }
            Token::Eof => Err(ParserError {
                message: "No value was found, end of file instead".to_string(),
                line_no: self.lexer.line_no,
                byte_offset: self.lexer.byte_offset,
            }),
            _ => Err(ParserError {
                message: "Invalid token for object".to_string(),
                line_no: self.lexer.line_no,
                byte_offset: self.lexer.byte_offset,
            }),
        }
    }

    fn object(&mut self) -> Result<HashMap<String, JsonValue>, ParserError> {
        let mut object = HashMap::new();
        loop {
            // handle object key
            self.lexer.get_next_token()?;
            let key = match &self.lexer.token {
                Token::String(value) => value.clone(),
                Token::RightBrace => {
                    if object.is_empty() {
                        return Ok(object);
                    } else {
                        return Err(ParserError {
                            message: "String required for object key".to_string(),
                            line_no: self.lexer.line_no,
                            byte_offset: self.lexer.byte_offset,
                        });
                    }
                }
                Token::Eof => {
                    return Err(ParserError {
                        message: "Object never closed, \"}\" is missing".to_string(),
                        line_no: self.lexer.line_no,
                        byte_offset: self.lexer.byte_offset,
                    });
                }
                _ => {
                    return Err(ParserError {
                        message: "String required for object key".to_string(),
                        line_no: self.lexer.line_no,
                        byte_offset: self.lexer.byte_offset,
                    });
                }
            };

            // handle colon
            self.lexer.get_next_token()?;
            if self.lexer.token != Token::Colon {
                return Err(ParserError {
                    message: "Colon required after object key".to_string(),
                    line_no: self.lexer.line_no,
                    byte_offset: self.lexer.byte_offset,
                });
            }

            // handle object value
            self.lexer.get_next_token()?;
            let value = self.parse_value()?;

            // add key, value to object
            object.insert(key, value);

            // either there is a comma and another value is expected
            // or there is a } and it's over
            self.lexer.get_next_token()?;
            match &self.lexer.token {
                Token::Comma => continue,
                Token::RightBrace => break,
                Token::Eof => {
                    return Err(ParserError {
                        message: "Object never closed, \"}\" is missing".to_string(),
                        line_no: self.lexer.line_no,
                        byte_offset: self.lexer.byte_offset,
                    });
                }
                _ => {
                    return Err(ParserError {
                        message: "Comma required between object entries".to_string(),
                        line_no: self.lexer.line_no,
                        byte_offset: self.lexer.byte_offset,
                    });
                }
            }
        }
        Ok(object)
    }

    fn array(&mut self) -> Result<Vec<JsonValue>, ParserError> {
        let mut array = Vec::<JsonValue>::new();
        self.lexer.get_next_token()?;

        if self.lexer.token == Token::RightBracket {
            return Ok(array);
        }

        loop {
            // handle value
            let value = self.parse_value()?;
            array.push(value);

            // either there is a comma and another value is expected
            // or there is a ] and it's over
            self.lexer.get_next_token()?;
            match &self.lexer.token {
                Token::Comma => {
                    self.lexer.get_next_token()?;
                    continue;
                }
                Token::RightBracket => break,
                Token::Eof => {
                    return Err(ParserError {
                        message: "Array never closed, \"]\" is missing".to_string(),
                        line_no: self.lexer.line_no,
                        byte_offset: self.lexer.byte_offset,
                    });
                }
                _ => {
                    return Err(ParserError {
                        message: "Comma required between array values".to_string(),
                        line_no: self.lexer.line_no,
                        byte_offset: self.lexer.byte_offset,
                    });
                }
            }
        }
        Ok(array)
    }
}

// UNIT TESTS

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_str(input: &str) -> Result<JsonValue, ParserError> {
        let lines: Vec<Result<String, Error>> = input.lines().map(|l| Ok(l.to_string())).collect();
        let lexer = lexer::Lexer::new(lines.into_iter());
        Parser::new(lexer).parse()
    }

    fn ok(input: &str) -> JsonValue {
        match parse_str(input) {
            Ok(v) => v,
            Err(e) => panic!("expected Ok for {:?}, got error: {}", input, e),
        }
    }

    fn err_message(input: &str) -> String {
        match parse_str(input) {
            Ok(v) => panic!("expected Err for {:?}, got Ok({})", input, v),
            Err(e) => e.message,
        }
    }

    fn obj(entries: Vec<(&str, JsonValue)>) -> JsonValue {
        JsonValue::Object(
            entries
                .into_iter()
                .map(|(k, v)| (k.to_string(), v))
                .collect(),
        )
    }

    // ---------- scalars ----------

    #[test]
    fn parses_null() {
        assert!(ok("null") == JsonValue::Null);
    }

    #[test]
    fn parses_booleans() {
        assert!(ok("true") == JsonValue::Bool(true));
        assert!(ok("false") == JsonValue::Bool(false));
    }

    #[test]
    fn parses_integers() {
        assert!(ok("0") == JsonValue::Number(0.0));
        assert!(ok("42") == JsonValue::Number(42.0));
        assert!(ok("-7") == JsonValue::Number(-7.0));
    }

    #[test]
    fn parses_floats_and_exponents() {
        assert!(ok("3.5") == JsonValue::Number(3.5));
        assert!(ok("-0.25") == JsonValue::Number(-0.25));
        assert!(ok("1e3") == JsonValue::Number(1000.0));
        assert!(ok("2.5E-1") == JsonValue::Number(0.25));
    }

    #[test]
    fn parses_strings() {
        assert!(ok("\"hello\"") == JsonValue::String("hello".to_string()));
        assert!(ok("\"\"") == JsonValue::String(String::new()));
        assert!(ok("\"with space\"") == JsonValue::String("with space".to_string()));
    }

    // ---------- arrays ----------

    #[test]
    fn parses_empty_array() {
        assert!(ok("[]") == JsonValue::Array(vec![]));
    }

    #[test]
    fn parses_array_of_numbers() {
        assert!(
            ok("[1, 2, 3]")
                == JsonValue::Array(vec![
                    JsonValue::Number(1.0),
                    JsonValue::Number(2.0),
                    JsonValue::Number(3.0),
                ])
        );
    }

    #[test]
    fn parses_mixed_array() {
        assert!(
            ok("[null, true, 1, \"a\"]")
                == JsonValue::Array(vec![
                    JsonValue::Null,
                    JsonValue::Bool(true),
                    JsonValue::Number(1.0),
                    JsonValue::String("a".to_string()),
                ])
        );
    }

    #[test]
    fn parses_nested_arrays() {
        assert!(
            ok("[[1], [], [2, [3]]]")
                == JsonValue::Array(vec![
                    JsonValue::Array(vec![JsonValue::Number(1.0)]),
                    JsonValue::Array(vec![]),
                    JsonValue::Array(vec![
                        JsonValue::Number(2.0),
                        JsonValue::Array(vec![JsonValue::Number(3.0)]),
                    ]),
                ])
        );
    }

    #[test]
    fn parses_array_without_whitespace() {
        assert!(
            ok("[1,2]") == JsonValue::Array(vec![JsonValue::Number(1.0), JsonValue::Number(2.0)])
        );
    }

    // ---------- objects ----------

    #[test]
    fn parses_empty_object() {
        assert!(ok("{}") == obj(vec![]));
    }

    #[test]
    fn parses_simple_object() {
        assert!(ok("{\"a\": 1}") == obj(vec![("a", JsonValue::Number(1.0))]));
    }

    #[test]
    fn parses_object_with_multiple_entries() {
        assert!(
            ok("{\"a\": 1, \"b\": true, \"c\": null}")
                == obj(vec![
                    ("a", JsonValue::Number(1.0)),
                    ("b", JsonValue::Bool(true)),
                    ("c", JsonValue::Null),
                ])
        );
    }

    #[test]
    fn parses_nested_object() {
        assert!(
            ok("{\"outer\": {\"inner\": [1, 2]}}")
                == obj(vec![(
                    "outer",
                    obj(vec![(
                        "inner",
                        JsonValue::Array(vec![JsonValue::Number(1.0), JsonValue::Number(2.0)])
                    )])
                )])
        );
    }

    #[test]
    fn parses_object_containing_empty_containers() {
        assert!(
            ok("{\"a\": {}, \"b\": []}")
                == obj(vec![("a", obj(vec![])), ("b", JsonValue::Array(vec![]))])
        );
    }

    #[test]
    fn duplicate_keys_last_value_wins() {
        assert!(ok("{\"a\": 1, \"a\": 2}") == obj(vec![("a", JsonValue::Number(2.0))]));
    }

    #[test]
    fn parses_array_of_objects() {
        assert!(
            ok("[{\"a\": 1}, {\"a\": 2}]")
                == JsonValue::Array(vec![
                    obj(vec![("a", JsonValue::Number(1.0))]),
                    obj(vec![("a", JsonValue::Number(2.0))]),
                ])
        );
    }

    // ---------- multi-line input ----------

    #[test]
    fn parses_multiline_document() {
        let input = "{\n  \"name\": \"x\",\n  \"tags\": [\n    1,\n    2\n  ]\n}";
        assert!(
            ok(input)
                == obj(vec![
                    ("name", JsonValue::String("x".to_string())),
                    (
                        "tags",
                        JsonValue::Array(vec![JsonValue::Number(1.0), JsonValue::Number(2.0)])
                    ),
                ])
        );
    }

    // ---------- error cases ----------

    #[test]
    fn empty_input_is_an_error() {
        assert!(err_message("").contains("No value was found"));
    }

    #[test]
    fn multiple_root_values_is_an_error() {
        assert!(err_message("1 2").contains("multiple root values"));
        assert!(err_message("{} []").contains("multiple root values"));
    }

    #[test]
    fn unclosed_array_is_an_error() {
        assert!(err_message("[1, 2").contains("Array never closed"));
    }

    #[test]
    fn lone_left_bracket_is_an_error() {
        assert!(parse_str("[").is_err());
    }

    #[test]
    fn missing_comma_in_array_is_an_error() {
        assert!(err_message("[1 2]").contains("Comma required between array values"));
    }

    #[test]
    fn trailing_comma_in_array_is_an_error() {
        assert!(parse_str("[1, 2,]").is_err());
    }

    #[test]
    fn leading_comma_in_array_is_an_error() {
        assert!(parse_str("[,1]").is_err());
    }

    #[test]
    fn unclosed_object_is_an_error() {
        assert!(err_message("{\"a\": 1").contains("Object never closed"));
    }

    #[test]
    fn lone_left_brace_is_an_error() {
        assert!(err_message("{").contains("Object never closed"));
    }

    #[test]
    fn non_string_object_key_is_an_error() {
        assert!(err_message("{1: 2}").contains("String required for object key"));
    }

    #[test]
    fn missing_colon_is_an_error() {
        assert!(err_message("{\"a\" 1}").contains("Colon required after object key"));
    }

    #[test]
    fn missing_comma_in_object_is_an_error() {
        assert!(
            err_message("{\"a\": 1 \"b\": 2}").contains("Comma required between object entries")
        );
    }

    #[test]
    fn trailing_comma_in_object_is_an_error() {
        assert!(parse_str("{\"a\": 1,}").is_err());
    }

    #[test]
    fn missing_object_value_is_an_error() {
        assert!(parse_str("{\"a\": }").is_err());
    }

    #[test]
    fn stray_closing_tokens_are_errors() {
        assert!(parse_str("]").is_err());
        assert!(parse_str("}").is_err());
        assert!(parse_str(",").is_err());
        assert!(parse_str(":").is_err());
    }

    // ---------- Display ----------

    #[test]
    fn display_scalars() {
        assert_eq!(JsonValue::Null.to_string(), "null");
        assert_eq!(JsonValue::Bool(true).to_string(), "true");
        assert_eq!(JsonValue::Number(1.5).to_string(), "1.5");
        assert_eq!(JsonValue::String("hi".to_string()).to_string(), "\"hi\"");
    }

    #[test]
    fn display_array() {
        let v = JsonValue::Array(vec![JsonValue::Number(1.0), JsonValue::Null]);
        assert_eq!(v.to_string(), "[1, null]");
    }

    #[test]
    fn display_empty_containers() {
        assert_eq!(JsonValue::Array(vec![]).to_string(), "[]");
        assert_eq!(obj(vec![]).to_string(), "{}");
    }

    #[test]
    fn display_single_entry_object() {
        let v = obj(vec![("a", JsonValue::Bool(false))]);
        assert_eq!(v.to_string(), "{\"a\": false}");
    }

    #[test]
    fn display_round_trips_through_parser() {
        let original = ok("{\"k\": [1, 2, {\"z\": null}]}");
        let reparsed = ok(&original.to_string());
        assert!(original == reparsed);
    }

    // ---------- ParserError Display ----------

    #[test]
    fn parser_error_display_format() {
        let e = ParserError {
            message: "boom".to_string(),
            line_no: 3,
            byte_offset: 9,
        };
        assert_eq!(e.to_string(), "Parser Error at line 3 character 9 : boom");
    }
}

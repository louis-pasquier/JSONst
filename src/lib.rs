//! A lightweight JSON parsing library.
//!
//! This crate provides straightforward utilities to parse JSON data from both
//! in-memory strings and file streams into a [`JsonValue`] representation.
//!
//! # Examples
//!
//! Parsing from a string:
//! ```
//! use jsonst::parse_str;
//!
//! let json = r#"{"key": "value"}"#;
//! let parsed = parse_str(json).unwrap();
//! ```

mod common;
mod lexer;
mod parser;

pub use crate::common::{FileParseError, ParserError};
pub use crate::parser::JsonValue;

use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;

/// Parses a JSON document directly from a file path.
///
/// This function opens the file and reads it line-by-line using a `BufReader`,
/// which is memory efficient for large JSON files as it doesn't load the entire
/// file into memory at once.
///
/// # Arguments
///
/// * `filename` - A path to the file containing the JSON data.
///
/// # Errors
///
/// This function will return a [`FileParseError`] if:
/// * The file does not exist or cannot be opened (I/O error).
/// * The file contents are not valid JSON (Parsing error).
///
/// # Examples
///
/// ```no_run
/// use jsonst::parse_file;
///
/// let parsed = parse_file("data.json").expect("Failed to parse file");
/// ```
pub fn parse_file<P>(filename: P) -> Result<JsonValue, FileParseError>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    let lines = io::BufReader::new(file).lines();

    let scanner = lexer::Lexer::new(lines);
    let mut parser = parser::Parser::new(scanner);
    let result = parser.parse()?;

    Ok(result)
}

/// Parses a JSON document from a string slice.
///
/// # Arguments
///
/// * `json` - A string slice containing the raw JSON payload.
///
/// # Errors
///
/// This function will return a [`ParserError`] if the provided string
/// is not valid JSON (e.g., missing quotes, trailing commas, or syntax errors).
///
/// # Examples
///
/// ```
/// use jsonst::parse_str;
///
/// let payload = r#"[1, 2, 3, {"nested": true}]"#;
/// match parse_str(payload) {
///     Ok(value) => println!("Successfully parsed: {:?}", value),
///     Err(e) => eprintln!("Failed to parse: {:?}", e),
/// }
/// ```
pub fn parse_str(json: &str) -> Result<JsonValue, ParserError> {
    let lines = json.lines().map(|line| Ok(line.to_string()));

    let scanner = lexer::Lexer::new(lines);
    let mut parser = parser::Parser::new(scanner);
    let result = parser.parse()?;

    Ok(result)
}

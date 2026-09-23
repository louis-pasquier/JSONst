mod lexer;

use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;

pub fn parse_file<P>(filename: P) -> io::Result<()>
where P: AsRef<Path>, {
    let file = File::open(filename)?;
    let lines = io::BufReader::new(file).lines();
    lexer::parse(lines);
    // let mut scanner = Lexer::new(lines);
    // TODO : parser call
    Ok(())
}

pub fn parse_str(json: &str) -> io::Result<()> {
    let lines = json.lines().map(|line| Ok(line.to_string()));
    lexer::parse(lines);
    // let mut scanner = Lexer::new(lines);
    // TODO : parser call
    Ok(())
}



mod scanner;
mod token;


use scanner::Scanner;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;

use token::Token;

fn main() {
    println!("Running JSONst");

    let lines = read_lines("./samples/list.json").expect("Failed to read file");
    let mut scanner = Scanner::new(lines);
    //scanner.parse_all_chars();
    while scanner.token != Token::Eof {
        scanner.get_next_token();
        println!("{:?}", scanner.token);
    }

}

// The output is wrapped in a Result to allow matching on errors.
// Returns an Iterator to the Reader of the lines of the file.
fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where P: AsRef<Path>, {
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}
# JSONst

A JSON parser written from scratch in Rust. It reads input line by line, tokenizes it with a hand-written lexer, and builds a tree of `JsonValue`s with a recursive-descent parser.

It was built to learn how lexers and parsers work, so the code favors clarity over speed.

## How it works

```
 input lines ──► Lexer ──► Token stream ──► Parser ──► JsonValue
```

| Module | Role |
| --- | --- |
| `lexer` | Reads characters and produces one `Token` at a time (`{`, `}`, `[`, `]`, `,`, `:`, strings, numbers, literals, EOF) |
| `common` | The `Token` and `ParserError` types |
| `parser` | Recursive-descent parser that turns tokens into a `JsonValue` or a `ParserError` |

The parser pulls tokens from the lexer on demand using a single token of lookahead, so the input is never held in memory all at once.

## Usage

```rust
use json_st::parse_file;

fn main() {
    if let Ok(json) = parse_file("./samples/list.json") {
        println!("{}", json)
    }
}
```

### The value type

```rust
pub enum JsonValue {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<JsonValue>),
    Object(HashMap<String, JsonValue>),
}
```

### Error format

```
Parser Error at line 3 character 9 : Comma required between array values
```

`character` is a byte offset within the line.

## Building and testing

```sh
cargo build
cargo test
```

## Known limitations

- Numbers are stored as `f64`, so integers above 2^53 lose precision and values like `1e999` overflow
- Objects use a `HashMap`, so key order is not preserved
- There is no nesting depth limit, so extremely deep input can overflow the stack
- Error positions point to the end of the offending token rather than its start
- Whitespace and string validation are looser than the JSON spec in a few places
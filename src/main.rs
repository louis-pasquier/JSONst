use json_st::parse_file;

fn main() {
    println!("Running JSONst");

    if let Ok(json) = parse_file("./samples/list.json") {
        println!("{}", json)
    }
}
